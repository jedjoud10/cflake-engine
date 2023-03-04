use crate::{Graphics, StagingView, StagingViewWrite, GpuPod, GpuPodRelaxed};
use parking_lot::Mutex;
use std::{num::NonZeroU64, ops::DerefMut, sync::{Arc, atomic::Ordering}, marker::PhantomData};
use utils::{ConcVec, AtomicBitSet};
use wgpu::{CommandEncoder, Maintain, MapMode, Buffer, Texture, Extent3d, Origin3d, TextureAspect, ImageDataLayout};

// Target for writing / download operations
// Either a wgpu buffer or a wgpu texture
pub enum StagingTarget<'a> {
    // Read/write from/to this texture
    // source layout must be divisible by COPY_BYTES_PER_ROW_ALIGNMENT
    Texture {
        texture: &'a Texture,
        offset: Extent3d,
        mip_level: u32,
        origin: Origin3d,
        aspect: TextureAspect,
        data_layout: ImageDataLayout,
        stride: u64,
    },


    // Read/write from/to this buffer
    Buffer {
        buffer: &'a Buffer,
        offset: u64,
        size: u64,
    },
}

// Helper struct that will temporarily store mapped buffers so we can have
// StagingView / StagingViewMut that we can read and write from 
// This will re-use unmapped buffers to avoid many many buffer creations
pub(crate) struct StagingPool {
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
        mode: MapMode
    ) -> (usize, &Buffer) {
        // Usages for map reading and map writing
        let read = wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST;
        let write = wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC;

        log::debug!("Looking for staging buffer [cap = {capacity}b, mode = {mode:?}]");

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
                label: None,
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

    /*
    // Map a target for writing only (maps an intermediate staging buffer)
    // Src target must have the COPY_SRC buffer usage flag
    pub fn map_read<'a>(
        &'a self,
        target: StagingTarget,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingView<'a>> {
        None
    }

    // Map a target for writing only (maps an intermediate staging buffer)
    // Src target must have the COPY_DST buffer usage flag
    pub fn map_write<'a>(
        &'a self,
        target: StagingTarget,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingViewWrite<'a>> {
        None
    }
    */

    // Writes to the destination target using the source byte buffer
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple write calls and expect them to be batched together
    pub fn write<'a>(
        &'a self,
        graphics: &Graphics,
        target: StagingTarget,
        src: &[u8],
    ) {
        match target {
            // Handle buffer writing
            StagingTarget::Buffer { buffer, offset, size } => {
                debug_assert_eq!(size as usize, src.len());
                graphics.queue().write_buffer(buffer, offset, src);
            },
            
            // Handle texture writing
            _ => {
            }
        }
    }

    // Reads the given staging target into the destination buffer
    // Will stall the CPU, since this is waiting for GPU data
    // TODO: Make it submit the current recorder only when it was written to recently
    pub fn read<'a>(
        &'a self,
        graphics: &'a Graphics,
        target: StagingTarget,
        dst: &mut [u8],
    ) {
        match target {
            // Handle buffer writing
            StagingTarget::Buffer { buffer, offset, size } => {
                assert_eq!(size as usize, dst.len());
                assert!(buffer.usage().contains(wgpu::BufferUsages::COPY_SRC));

                // Get a encoder (reused or not to perform a copy)
                let mut encoder = graphics.acquire();
                let (i, staging) = self.find_or_allocate(graphics, size, MapMode::Read);

                // Copy to staging first
                encoder.copy_buffer_to_buffer(
                    buffer,
                    offset,
                    staging,
                    0,
                    size
                );

                // Put the encoder back into the cache, and submit ALL encoders
                graphics.reuse([encoder]);
                // (Also wait for their subbmission)
                graphics.submit_unused(true);

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
            },
        
            // Handle texture writing
            _ => { todo!() }
        }
    }
}
