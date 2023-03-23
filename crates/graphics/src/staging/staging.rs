use crate::{GpuPod, Graphics, StagingView, StagingViewWrite};
use parking_lot::Mutex;
use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU64},
    ops::DerefMut,
    sync::{atomic::Ordering, Arc},
};
use utils::{AtomicBitSet, ConcVec};
use wgpu::{
    Buffer, CommandEncoder, Extent3d, ImageDataLayout, Maintain,
    MapMode, Origin3d, Texture, TextureAspect,
};

// Helper struct that will temporarily store mapped buffers so we can have
// StagingView / StagingViewMut that we can read and write from
// This will re-use unmapped buffers to avoid many many buffer creations
pub struct StagingPool {
    // Keeps track of mapping buffers
    pub(crate) allocations: Arc<ConcVec<Buffer>>,

    // Keeps track of the mapping state
    pub(crate) states: AtomicBitSet,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(ConcVec::new()),
            states: AtomicBitSet::new(),
        }
    }

    // Tries to find a free buffer that we can use
    // Might allocate more buffers than needed based on capacity
    fn find_or_allocate(
        &self,
        graphics: &Graphics,
        capacity: u64,
        mode: MapMode,
    ) -> (usize, &Buffer) {
        // Usages for map reading and map writing
        let read = wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::COPY_DST;
        let write = wgpu::BufferUsages::MAP_WRITE
            | wgpu::BufferUsages::COPY_SRC;

        // Try to find a free buffer
        // If that's not possible, simply create a new one
        self.allocations.iter().enumerate().find(|(i, buffer)| {
            let cap = buffer.size() >= capacity;
            let mode = match mode {
                MapMode::Read => buffer.usage().contains(read),
                MapMode::Write => buffer.usage().contains(write),
            };
            let used = !self.states.get(*i, Ordering::Relaxed);
            cap && mode && used
        }).unwrap_or_else(|| {
            log::warn!("Did not find staging buffer with the proper requirements [cap = {capacity}b, mode = {mode:?}]");
            
            // Scale up the capacity (so we don't have to allocate a new block anytime soon)
            let capacity = (capacity * 4).next_power_of_two();
            let capacity = capacity.max(256);

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
            log::warn!("Allocating new staging buffer [cap = {capacity}b, mode = {mode:?}]");
            let index = self.allocations.push(buffer);

            // Also add the "used" buffer state to the state tracker
            self.states.set(index, Ordering::Relaxed);
            (index, &self.allocations[index])
        })
    }
}

impl StagingPool {
    // Map a target for reading only (maps an intermediate staging buffer)
    // Src target must have the COPY_SRC buffer usage flag
    pub fn map_buffer_read<'a>(
        &'a self,
        graphics: &'a Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
    ) -> Option<StagingView<'a>> {
        assert!(buffer
            .usage()
            .contains(wgpu::BufferUsages::COPY_SRC));
        log::trace!("map_buffer_read: offset: {offset}, size: {size}");

        // Get a encoder (reused or not to perform a copy)
        let mut encoder = graphics.acquire();
        let (i, staging) =
            self.find_or_allocate(graphics, size, MapMode::Read);

        // Copy to staging first
        encoder
            .copy_buffer_to_buffer(buffer, offset, staging, 0, size);

        // Put the encoder back into the cache, and submit ALL encoders
        graphics.reuse([encoder]);
        // (Also wait for their subbmission)
        graphics.submit(true);

        // Map the staging buffer
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

        // Map synchronously
        let slice = staging.slice(0..size);
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap()
        });
        graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then read from the buffer
        if let Ok(Ok(_)) = rx.recv() {
            return Some(StagingView {
                index: i,
                states: &&self.states,
                staging,
                view: Some(slice.get_mapped_range()),
            });
        } else {
            panic!("Could not receive read map async")
        }
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
        log::trace!("map_buffer_write: offset: {offset}, size: {size}");
        let size = NonZeroU64::new(size);
        let write = graphics.queue().write_buffer_with(
            buffer,
            offset,
            size.unwrap(),
        )?;
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
        debug_assert_eq!(size as usize, src.len());
        log::trace!("write_buffer: offset: {offset}, size: {size}");
        graphics.queue().write_buffer(buffer, offset, src);
    }

    // Reads the given buffer into the destination buffer
    // Will stall the CPU, since this is waiting for GPU data
    // TODO: Make it submit the current recorder only when it was written to recently
    pub fn read_buffer<'a>(
        &'a self,
        graphics: &'a Graphics,
        buffer: &Buffer,
        offset: u64,
        size: u64,
        dst: &mut [u8],
    ) {
        assert_eq!(size as usize, dst.len());
        assert!(buffer
            .usage()
            .contains(wgpu::BufferUsages::COPY_SRC));
        log::trace!("read_buffer: offset: {offset}, size: {size}");

        // Get a encoder (reused or not to perform a copy)
        let mut encoder = graphics.acquire();
        let (i, staging) =
            self.find_or_allocate(graphics, size, MapMode::Read);

        // Copy to staging first
        encoder
            .copy_buffer_to_buffer(buffer, offset, staging, 0, size);

        // Put the encoder back into the cache, and submit ALL encoders
        graphics.reuse([encoder]);
        // (Also wait for their subbmission)
        graphics.submit(true);

        // Map the staging buffer
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

        // Map synchronously
        let slice = staging.slice(0..size);
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap()
        });
        graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then read from the buffer
        if let Ok(Ok(_)) = rx.recv() {
            dst.copy_from_slice(&slice.get_mapped_range());
            staging.unmap();
        } else {
            panic!("Could not receive read map async")
        }

        // Reset the state of the staging buffer
        self.states.remove(i, Ordering::Relaxed);
    }
}

impl StagingPool {
    // Map an texture for reading only (maps an intermediate staging buffer)
    // Src texture must have the COPY_SRC texture usage flag
    pub fn map_texture_read<'a>(
        &'a self,
        graphics: &'a Graphics,
        texture: &wgpu::Texture,
    ) -> Option<StagingView<'a>> {
        todo!()
    }

    // Map a texture for writing only
    // This is a "fire and forget" command that does not stall the CPU
    pub fn map_texture_write<'a>(
        &'a self,
        graphics: &'a Graphics,
        texture: &'a wgpu::Texture,
    ) -> Option<StagingViewWrite<'a>> {
        todo!()
    }

    // Writes to the destination texture using the source byte buffer
    // This is a "fire and forget" command that does not stall the CPU
    pub fn write_texture<'a>(
        &'a self,
        graphics: &Graphics,
        texture: &wgpu::Texture,
        mip_level: u32,
        origin: wgpu::Origin3d,
        extent: wgpu::Extent3d,
        aspect: wgpu::TextureAspect,
        offset: u64,
        bytes_per_row: Option<NonZeroU32>,
        rows_per_image: Option<NonZeroU32>,
        src: &[u8],
    ) {
        let image_copy_texture = wgpu::ImageCopyTexture {
            texture,
            mip_level,
            origin,
            aspect,
        };

        let data_layout = wgpu::ImageDataLayout {
            offset,
            bytes_per_row,
            rows_per_image,
        };

        graphics.queue().write_texture(
            image_copy_texture,
            src,
            data_layout,
            extent,
        );
    }

    // Reads the given buffer into the destination buffer
    // Will stall the CPU, since this is waiting for GPU data
    // TODO: Make it submit the current recorder only when it was written to recently
    pub fn read_texture<'a>(
        &'a self,
        graphics: &'a Graphics,
        texture: &wgpu::Texture,
        dst: &mut [u8],
    ) {
    }
}
