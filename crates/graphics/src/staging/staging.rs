use crate::{
    GpuPod, Graphics, StagingView, StagingViewWrite, TextureStagingView, TextureStagingViewWrite,
};
use parking_lot::Mutex;
use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU64},
    ops::DerefMut,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use utils::{AtomicBitSet, ConcVec};
use vek::num_integer::Integer;
use wgpu::{
    Buffer, CommandEncoder, Extent3d, ImageDataLayout, Maintain, MapMode, Origin3d, Texture,
    TextureAspect,
};

// Helper struct that will temporarily store mapped buffers so we can have
// StagingView / StagingViewMut that we can read and write from
// This will re-use unmapped buffers to avoid many buffer creations
// Allows us to map and read/write buffers and textures
// TODO: Make it submit the current recorder only when it was written to recently
pub struct StagingPool {
    // Keeps track of mapping buffers
    pub(crate) allocations: Arc<ConcVec<Buffer>>,

    // Keeps track of the mapping state
    pub(crate) used: Arc<AtomicBitSet<AtomicUsize>>,

    // Keeps track of the buffers that we *must* unmap
    // Only used for ASYNC readback buffers
    pub(crate) must_unmap: Arc<AtomicBitSet<AtomicUsize>>,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(ConcVec::new()),
            used: Arc::new(AtomicBitSet::new()),
            must_unmap: Arc::new(AtomicBitSet::new()),
        }
    }

    // Called at the end of every frame
    pub(crate) fn refresh(&self) {
        for (offset, chunk) in self.must_unmap.chunks().into_iter().enumerate() {
            let old = chunk.swap(0, Ordering::Relaxed);

            for i in 0..(usize::BITS) {
                if (old >> i) & 1 == 1 {
                    let index = i as usize + (offset * usize::BITS as usize);
                    self.allocations[index].unmap();
                    self.used.remove(index, Ordering::Relaxed);
                }
            }
        }
    }

    // Tries to find a free buffer that we can use
    // Might allocate more buffers than needed based on capacity
    pub(crate) fn find_or_allocate(
        &self,
        graphics: &Graphics,
        capacity: u64,
        mode: MapMode,
    ) -> (usize, &Buffer) {
        // Usages for map reading and map writing
        let read = wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST;
        let write = wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC;

        // Try to find a free buffer
        // If that's not possible, simply create a new one
        self.allocations
            .iter()
            .enumerate()
            .find(|(i, buffer)| {
                let cap = buffer.size() >= capacity;
                let mode = match mode {
                    MapMode::Read => buffer.usage().contains(read),
                    MapMode::Write => buffer.usage().contains(write),
                };
                let free = !self.used.get(*i, Ordering::Relaxed);
                cap && mode && free
            })
            .unwrap_or_else(|| {
                //log::trace!("did not find staging buffer with the proper requirements [cap = {capacity}b, mode = {mode:?}]");

                // Scale up the capacity (so we don't have to allocate a new block anytime soon)
                let capacity = (capacity * 4).next_power_of_two();
                let capacity = capacity
                    .max(256)
                    .min(graphics.device().limits().max_buffer_size);

                // Create the buffer descriptor for a new buffer
                let desc = wgpu::BufferDescriptor {
                    label: Some("graphics-staging-buffer"),
                    size: capacity,
                    usage: match mode {
                        MapMode::Read => read,
                        MapMode::Write => write,
                    },
                    mapped_at_creation: false,
                };

                // Create the new buffer
                let buffer = graphics.device().create_buffer(&desc);
                //log::trace!("allocating new staging buffer [cap = {capacity}b, mode = {mode:?}]");
                let index = self.allocations.push(buffer);
                (index, &self.allocations[index])
            })
    }
}

impl StagingPool {
    // Map a buffer for reading only (maps an intermediate staging buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn map_buffer_read<'a>(
        &'a self,
        graphics: &'a Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
    ) -> StagingView<'a> {
        //log::trace!("map buffer read: offset: \n{offset}\nsize: {size}");

        // Get a encoder (reused or not to perform a copy)
        let (i, staging) = self.find_or_allocate(graphics, size, MapMode::Read);
        self.used.set(i, Ordering::Relaxed);

        // Copy to staging first
        let mut encoder = graphics.acquire();
        encoder.copy_buffer_to_buffer(buffer, offset, staging, 0, size);
        graphics.reuse([encoder]);
        graphics.submit(false);

        // Read the staging buffer
        let view = super::read_staging_buffer_view(graphics, staging, 0, size);

        StagingView {
            index: i,
            used: &self.used,
            staging,
            view: Some(view),
        }
    }

    // Map a buffer for asynchronous reading only
    // Src target must have the COPY_SRC buffer usage flag
    // The mapping of the buffer will not occur immediately
    pub fn map_buffer_read_async(
        &self,
        graphics: &Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
        callback: impl FnOnce(&[u8]) + Send + 'static,
    ) {
        //log::trace!("map buffer read sync: offset: \n{offset}\nsize: {size}");

        // Get a encoder (reused or not to perform a copy)
        let (i, staging) = self.find_or_allocate(graphics, size, MapMode::Read);
        self.used.set(i, Ordering::Relaxed);

        // Copy to staging first
        let mut encoder = graphics.acquire();
        encoder.copy_buffer_to_buffer(buffer, offset, staging, 0, size);
        graphics.reuse([encoder]);
        graphics.submit(false);

        // Read the staging buffer asynchronously
        super::async_read_staging_buffer(
            self.allocations.clone(),
            self.must_unmap.clone(),
            i,
            0,
            size,
            callback,
        );
    }

    // Map a target for writing only
    // This is a "fire and forget" command that does not stall the CPU
    pub fn map_buffer_write<'a>(
        &'a self,
        graphics: &'a Graphics,
        buffer: &'a Buffer,
        offset: u64,
        size: u64,
    ) -> Option<StagingViewWrite<'a>> {
        //log::trace!("map buffer write: offset: \n{offset}\nsize: {size}");
        let size = NonZeroU64::new(size);
        let write = graphics
            .queue()
            .write_buffer_with(buffer, offset, size.unwrap())?;
        Some(StagingViewWrite { write })
    }

    // Writes to the destination buffer using the source byte buffer
    // This is a "fire and forget" command that does not stall the CPU
    pub fn write_buffer<'a>(
        &'a self,
        graphics: &Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
        src: &[u8],
    ) {
        //log::trace!("write buffer: offset: \n{offset}\nsize: {size}");
        graphics.queue().write_buffer(buffer, offset, src);
    }

    // Reads the given buffer into the destination buffer
    // Will stall the CPU, since this is waiting for GPU data
    pub fn read_buffer<'a>(
        &'a self,
        graphics: &'a Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
        dst: &mut [u8],
    ) {
        //log::trace!("read buffer: offset: \n{offset}\nsize: {size}");
        let view = self.map_buffer_read(graphics, buffer, offset, size);
        dst.copy_from_slice(view.as_ref());
    }
}

impl StagingPool {
    // Map an texture for reading only (maps an intermediate staging buffer)
    // Src texture must have the COPY_SRC texture usage flag
    pub fn map_texture_read<'a>(
        &'a self,
        graphics: &'a Graphics,
        image_copy_texture: wgpu::ImageCopyTexture,
        data_layout: wgpu::ImageDataLayout,
        extent: wgpu::Extent3d,
        size: u64,
    ) -> TextureStagingView<'a> {
        //log::trace!("map texture read: \nimage copy texture: {image_copy_texture:#?}\ndata layout: {data_layout:#?}\nextent: {extent:#?}");

        // Get a encoder (reused or not to perform a copy)
        let (i, staging) = self.find_or_allocate(graphics, size, MapMode::Read);
        self.used.set(i, Ordering::Relaxed);

        // Copy to staging first
        let mut encoder = graphics.acquire();
        assert!(data_layout
            .bytes_per_row
            .unwrap_or_default()
            .is_multiple_of(&256));
        encoder.copy_texture_to_buffer(
            image_copy_texture,
            wgpu::ImageCopyBuffer {
                buffer: staging,
                layout: data_layout,
            },
            extent,
        );
        graphics.reuse([encoder]);
        graphics.submit(false);

        // Read the staging buffer
        let view = super::read_staging_buffer_view(graphics, staging, 0, size);

        TextureStagingView {
            index: i,
            used: &self.used,
            staging,
            view: Some(view),
        }
    }

    // Map a texture for asynchronous reading only
    // Src target must have the COPY_SRC texture usage flag
    // The mapping of the texture will not occur immediately
    pub fn map_texture_read_async<'a>(
        &self,
        graphics: &Graphics,
        image_copy_texture: wgpu::ImageCopyTexture,
        data_layout: wgpu::ImageDataLayout,
        extent: wgpu::Extent3d,
        callback: impl FnOnce(&[u8]) + Sync + Send + 'static,
    ) {
        todo!()
    }

    // Map a texture for writing only
    // This is a "fire and forget" command that does not stall the CPU
    pub fn map_texture_write<'a>(
        &'a self,
        graphics: &'a Graphics,
        image_copy_texture: wgpu::ImageCopyTexture,
        data_layout: wgpu::ImageDataLayout,
        extent: wgpu::Extent3d,
    ) -> TextureStagingViewWrite<'a> {
        todo!()
    }

    // Writes to the destination texture using the source byte buffer
    // This is a "fire and forget" command that does not stall the CPU
    pub fn write_texture<'a>(
        &'a self,
        graphics: &Graphics,
        image_copy_texture: wgpu::ImageCopyTexture,
        data_layout: wgpu::ImageDataLayout,
        extent: wgpu::Extent3d,
        src: &[u8],
    ) {
        graphics
            .queue()
            .write_texture(image_copy_texture, src, data_layout, extent);
    }

    // Reads the given buffer into the destination buffer
    // Will stall the CPU, since this is waiting for GPU data
    pub fn read_texture<'a>(
        &'a self,
        graphics: &'a Graphics,
        image_copy_texture: wgpu::ImageCopyTexture,
        data_layout: wgpu::ImageDataLayout,
        extent: wgpu::Extent3d,
        dst: &mut [u8],
    ) {
        //log::trace!("map texture read: \nimage copy texture: {image_copy_texture:#?}\ndata layout: {data_layout:#?}\nextent: {extent:#?}");
        let view = self.map_texture_read(
            graphics,
            image_copy_texture,
            data_layout,
            extent,
            dst.len() as u64,
        );
        dst.copy_from_slice(view.as_ref());
    }
}
