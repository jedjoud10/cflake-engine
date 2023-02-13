use std::{num::NonZeroU64, sync::Arc};
use parking_lot::Mutex;
use wgpu::{CommandEncoder, Maintain};
use crate::Graphics;

// Contains a sub-allocated block of memory (which also contains a reference to it's backing WGPU buffer)
pub struct BlockId {
    buffer: Arc<wgpu::Buffer>,
    start: u64,
    end: u64, 
}

// A backed up allocation that might contain sparsely used data
struct Allocation {
    backing: Arc<wgpu::Buffer>,
    size: wgpu::BufferSize,
    used: Vec<BlockId>,
}

// Staging buffer belt that can re-use multiple mappable buffers for 
// multiple download / upload operations
pub struct StagingPool {
    download: Mutex<Vec<Allocation>>,
    chunk_size: u64,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new(chunk_size: u64) -> Self {
        Self {
            download: Mutex::new(Vec::new()),
            chunk_size,
        }
    }

    // Tries to find a free block of memory that we can use, and allocate a new one if needed
    pub fn find_or_allocate(
        &self,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferSize,
    ) -> BlockId {
        // Iterates over each block and checks if any of them have enough space for "capacity"
        let locked = self.download.lock();
        for allocation in locked.iter() {
            allocation.
        }
    }

    // Request an immediate buffer download (copies data from buffer to accessible mappable buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn download<'a>(
        buffer: &wgpu::Buffer,
        allocation: &'a StagingId,
        graphics: &Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
    ) -> Option<wgpu::BufferView<'a>> {
        // Decomspoe the allocation buffer
        let StagingId {
            buffer: staging,
            start,
            end,
        } = allocation;

        // Record the copy command
        let mut encoder = graphics.acquire();
        let slice = allocation.buffer.slice(start..end);
        encoder.copy_buffer_to_buffer(
            &buffer,
            offset,
            &staging,
            *start,
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
            Some(slice.get_mapped_range())
        } else {
            None
        }
    }

    // Request an immediate buffer upload by copying data into a buffer and return it
}