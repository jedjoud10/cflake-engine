use std::{num::NonZeroU64, sync::Arc, ops::DerefMut};
use parking_lot::Mutex;
use wgpu::{CommandEncoder, Maintain};
use crate::Graphics;

// Contains a sub-allocated block of memory (which also contains a reference to it's backing WGPU buffer)
pub struct BlockId {
    buffer: Arc<wgpu::Buffer>,
    start: u64,
    end: u64, 
}

// This is the view returned from the download() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
pub struct StagingView<'a> {
    view: wgpu::BufferView<'a>,
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view
    }
}

// This is the view returned from the upload() method of the staging pool
// This allows us to write to the given buffer (it will submit this write when this gets dropped)
pub struct StagingViewWrite<'a> {
    view: wgpu::BufferViewMut<'a>,
    buffer: &'a wgpu::Buffer,
    staging: &'a wgpu::Buffer,
    graphics: &'a Graphics,
    dst_offset: wgpu::BufferAddress,
    staging_offset: wgpu::BufferAddress,
    size: wgpu::BufferAddress,
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.view
    }
}

impl<'a> Drop for StagingViewWrite<'a> {
    fn drop(&mut self) {
        self.staging.unmap();

        // Record the copy command
        let mut encoder = self.graphics.acquire();
        encoder.copy_buffer_to_buffer(
            &self.staging,
            self.staging_offset,
            &self.buffer,
            self.dst_offset,
            self.size
        );  

        // Submit the copy command (but don't wait for it)
        self.graphics.submit(Some(encoder));
        self.graphics.device().poll(Maintain::Wait);
    }
}

// A backed up allocation that might contain sparsely used data
#[derive(Clone)]
struct Allocation {
    backing: Arc<wgpu::Buffer>,
    size: wgpu::BufferSize,
    read: bool,
    write: bool,
    used: Arc<Mutex<Vec<BlockId>>>,
}

// Staging buffer belt that can re-use multiple mappable buffers for 
// multiple download / upload operations
// TODO: Re-write this to make it a global buffer allocater / manager
pub struct StagingPool {
    allocations: Mutex<Vec<Allocation>>,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(Vec::new()),
        }
    }

    // Checks if a block has enough empty space to contain "capacity"
    // If we do, this returns a slice with the calculated (start, end) end points
    fn is_sparse_for(
        allocation: &Allocation,
        capacity: wgpu::BufferSize
    ) -> Option<(usize, usize)> {
        None
    }

    // Tries to find a free block of memory that we can use, and allocate a new one if needed
    fn allocate(
        &self,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferSize,
    ) -> &BlockId {
        // Iterates over each block and checks if any of them have enough space for "capacity"
        self.download. Self::is_sparse_for(allocation, capacity)
        
        // If we find a block with enough space, create a new BlockId and push it for that specific allocation

        // If not, make a new allocation (with a bigger capacity than the original allocation) and use a part of it
        todo!()
    }

    // Request an immediate buffer download (copies data from buffer to accessible mappable buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn download<'a>(
        &'a self,
        buffer: &wgpu::Buffer,
        graphics: &Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
    ) -> Option<StagingView<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            buffer: staging,
            start,
            end,
        } = self.allocate(wgpu::MapMode::Read, size);

        // Record the copy command
        let mut encoder = graphics.acquire();
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
        let slice = staging.slice(start..end);
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap()
        });
        graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then return
        if let Ok(Ok(_)) = rx.recv() {
            let view = slice.get_mapped_range();
            Some(StagingView {
                view,
            })
        } else {
            None
        }
    }

    // Request an immediate buffer upload by copying data into a buffer and return it
    // When the StagingViewMut is dropped, the data is copied to the staging buffer and to the original buffer
    pub fn upload<'a>(
        &'a self,
        buffer: &'a wgpu::Buffer,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
    ) -> Option<StagingViewWrite<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            buffer: staging,
            start,
            end,
        } = self.allocate(wgpu::MapMode::Write, size);

        // Map the staging buffer
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

        // Map async (but wait for submission)
        let slice = staging.slice(start..end);
        slice.map_async(wgpu::MapMode::Write, move |res| {
            tx.send(res).unwrap()
        });
        graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then return
        if let Ok(Ok(_)) = rx.recv() {
            let view = slice.get_mapped_range_mut();
            Some(StagingViewWrite {
                view,
                buffer,
                graphics,
                staging,
                dst_offset: offset,
                staging_offset: *start,
                size: size.get(),
            })
        } else {
            None
        }
    }
}