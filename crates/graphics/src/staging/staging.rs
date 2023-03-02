use crate::{Graphics, StagingView, StagingViewWrite, Buffer, GpuPod, GpuPodRelaxed};
use parking_lot::Mutex;
use std::{num::NonZeroU64, ops::DerefMut, sync::Arc, marker::PhantomData};
use utils::{ConcVec, AtomicBitSet};
use wgpu::{CommandEncoder, Maintain, MapMode, Buffer};

// Target for writing / download operations
// Either a wgpu buffer or a wgpu texture
pub enum StagingTarget<'a> {
    // Read/write from/to this texture
    // source layout must be divisible by COPY_BYTES_PER_ROW_ALIGNMENT
    Texture {
        texture: &'a wgpu::Texture,
        offset: wgpu::Extent3d,
        mip_level: u32,
        origin: wgpu::Origin3d,
        aspect: wgpu::TextureAspect,
        data_layout: wgpu::ImageDataLayout,
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
pub struct StagingPool {
    // Keeps track of mapping buffers
    allocations: Arc<ConcVec<Buffer>>, 

    // Keeps track of the mapping state
    states: AtomicBitSet,
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
    fn find_or_allocate(&self, capacity: u64, mode: MapMode) -> &Buffer {
        todo!()
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

    // Non-immediate write (enqueued and submitted in the next submit() call)
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

    // Non-immediate write (enqueued and submitted in the next submit() call)
    // Src target must have the COPY_DST buffer usage flag
    pub fn write<'a>(
        &'a self,
        graphics: &'a Graphics,
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
            _ => {}
        }
    }

    // Src target must have the COPY_SRC buffer usage flag
    pub fn read<'a>(
        &'a self,
        graphics: &'a Graphics,
        encoder: &'a mut CommandEncoder,
        target: StagingTarget,
        dst: &mut [u8],
    ) {
        match target {
            // Handle buffer writing
            StagingTarget::Buffer { buffer, offset, size } => {
                debug_assert_eq!(size as usize, dst.len());
                let staging = self.find_or_allocate(size, MapMode::Read);

                // Copy to staging first
                encoder.copy_buffer_to_buffer(
                    buffer,
                    offset,
                    staging,
                    0,
                    size
                );

                // Submit and read back the data
            },
        
            // Handle texture writing
            _ => {}
        }
    }
}
