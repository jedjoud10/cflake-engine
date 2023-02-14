use std::{num::NonZeroU64, sync::Arc, ops::DerefMut};
use parking_lot::Mutex;
use wgpu::{CommandEncoder, Maintain};
use crate::Graphics;

// Contains a sub-allocated block of memory (which also contains a reference to it's backing WGPU buffer)
pub struct BlockId<'a> {
    buffer: &'a wgpu::Buffer,
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

impl<'a> AsRef<[u8]> for StagingViewWrite<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.view
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
struct Allocation {
    backing: wgpu::Buffer,
    size: wgpu::BufferAddress,
    mode: wgpu::MapMode,
    used: Mutex<Vec<(wgpu::BufferAddress, wgpu::BufferAddress)>>,
}

impl Allocation {
    // Checks if a block has enough empty space to contain "capacity"
    // If we do, this returns a slice with the calculated (start, end) end points
    fn allocate_block(
        &self,
        capacity: wgpu::BufferAddress
    ) -> Option<BlockId> {
        // Keep track of empty spaces within the sub buffer
        let mut last = 0u64;
        let mut last_index = 0;
        let mut output = None;
        let mut used = self.used.lock(); 

        // Try to find a free block of memory within the used blocks
        for (i, (start, end)) in used.iter().enumerate() {
            if *start != last && (start - last) > capacity {
                output = Some((last, *start));
                last_index = i;
            }

            last = *end;
        }

        // Try to find a free block at the end of the used blocks of memory
        if let Some((_, end)) = used.last() {
            let potential_block_size = capacity - end;
            if output.is_none() && (potential_block_size > capacity) {
                output = Some((*end, *end + capacity));
                last_index = used.len()-1;
            }
        }

        // Try to find a free block at the start of the used blocks of memory
        if used.is_empty()
            && output.is_none()
            && (self.size >= capacity)
        {
            output = Some((0, capacity));
        }

        output.map(|(start, end)| {
            used.insert(last_index + 1,(start, end));
            BlockId {
                buffer: &self.backing,
                start,
                end,
            }
        })
    }
}

// Staging buffer belt that can re-use multiple mappable buffers for 
// multiple download / upload operations
// TODO: Re-write this to make it a global buffer allocater / manager
pub struct StagingPool {
    allocations: Vec<Allocation>,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
        }
    }

    // This WILL allocate a new block of memory for a specific size
    fn allocate(
        &self,
        graphics: &Graphics,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferAddress,
    ) -> Allocation {
        let usage = match mode {
            wgpu::MapMode::Read => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            wgpu::MapMode::Write => wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
        };

        // If not, make a new allocation (with a bigger capacity than the original allocation) and use a part of it
        let backing = graphics.device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: capacity,
            usage,
            mapped_at_creation: false,
        });

        Allocation {
            backing,
            size: capacity,
            mode,
            used: Mutex::new(Vec::new()),
        }
    }

    // Tries to find a free block of memory that we can use, and allocate a new one if needed
    fn find_or_allocate(
        &self,
        graphics: &Graphics,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferAddress,
    ) -> BlockId {
        // Iterates over each block and checks if any of them have enough space for "capacity"
        let block = self.allocations.iter().filter_map(|allocation| {
            allocation.allocate_block(capacity)
        }).next();

        if let Some(block) = block {
            block
        } else {
            //self.allocations.push(allocation);
            let allocation = self.allocate(graphics, mode, capacity);
            let allocation = self.allocations.get(0).unwrap();
            BlockId {
                buffer: &allocation.backing,
                start: todo!(),
                end: todo!(),
            }
        }
    }

    // Request an immediate buffer download (copies data from buffer to accessible mappable buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn download<'a>(
        &'a self,
        buffer: &wgpu::Buffer,
        graphics: &Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingView<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            buffer: staging,
            start,
            end,
        } = self.find_or_allocate(graphics, wgpu::MapMode::Read, size);

        // Record the copy command
        let mut encoder = graphics.acquire();
        encoder.copy_buffer_to_buffer(
            &buffer,
            offset,
            &staging,
            start,
            size
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
        size: wgpu::BufferAddress,
    ) -> Option<StagingViewWrite<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            buffer: staging,
            start,
            end,
        } = self.find_or_allocate(graphics, wgpu::MapMode::Write, size);

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
                staging_offset: start,
                size,
            })
        } else {
            None
        }
    }
}