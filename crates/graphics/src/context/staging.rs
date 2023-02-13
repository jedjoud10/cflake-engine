use std::num::NonZeroU64;
use wgpu::{CommandEncoder, Maintain};
use crate::Graphics;

// Allocation ID used to keep track of mapped accessible allocations
struct AllocationId {
    index: usize,
    start: u64,
    end: u64, 
}

// A backed up allocation that might contain sparsely used data
struct Allocation {
    backing: wgpu::Buffer,
    used: Vec<(u64, u64)>,
}

// Staging buffer belt that can re-use multiple mappable buffers for 
// multiple download / upload operations
pub struct StagingPool {
    download: Vec<Allocation>,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new(chunk_size: u64) -> Self {
        Self {
            download: todo!(),
        }
    }

    // Allocate enough space to be able to efficiently upload / download data from a buffer
    fn find_or_allocate(
        &self,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferSize,
    ) -> AllocationId {
        todo!()
    }

    // Request an immediate buffer download (copies data from buffer to accessible mappable buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn download<'a>(
        &'a self,
        graphics: &Graphics,
        buffer: &'a wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
    ) -> wgpu::BufferView<'a> {
        let AllocationId {
            index,
            start,
            end,
        } = self.find_or_allocate(wgpu::MapMode::Read, size);

        // Get the underlying staging buffer
        let allocation = &self.download[index];
        let staging = &allocation.backing;
        let slice = staging.slice(start..end);

        // Record the copy command
        let mut encoder = graphics.acquire();
        encoder.copy_buffer_to_buffer(
            &buffer,
            offset,
            &staging,
            start,
            size.get()
        );  

        // Submit the copy command (but don't wait for it)
        graphics.submit(Some(encoder));

        // Map the staging buffer
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

        // Map async (but wait for submission)
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap()
        });
        graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then return
        if let Ok(Ok(_)) = rx.recv() {
            return slice.get_mapped_range();
        } else {
            panic!()
        }
    }

    // Request an immediate buffer upload (either through mappable buffer or not)
    pub fn upload<'a>(
        &self, 
        graphics: &'a Graphics,
        buffer: &'a wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
    ) -> wgpu::QueueWriteBufferView<'a> {
        graphics.queue().write_buffer_with(
            buffer, offset, size
        ).unwrap()
    }
}