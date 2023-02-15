use std::{num::NonZeroU64, sync::Arc, ops::DerefMut};
use parking_lot::Mutex;
use utils::ConcVec;
use wgpu::{CommandEncoder, Maintain};
use crate::Graphics;

// Contains a sub-allocated block of memory (which also contains a reference to it's backing WGPU buffer)
struct BlockId<'a> {
    allocation: &'a Allocation,
    buffer: &'a wgpu::Buffer,
    start: u64,
    end: u64, 
}

// This is the view returned from the download() method of the staging pool
// This allows us to read the data of the given buffer at the given offset and slice
pub struct StagingView<'a> {
    allocation: &'a Allocation,
    graphics: &'a Graphics,
    block: BlockId<'a>,
    staging: &'a wgpu::Buffer,
    view: Option<wgpu::BufferView<'a>>,
}

impl<'a> Drop for StagingView<'a> {
    fn drop(&mut self) {
        drop(self.view.take().unwrap());
        self.staging.unmap();
        self.allocation.unmap_block(&self.block);
    }
}

impl<'a> AsRef<[u8]> for StagingView<'a> {
    fn as_ref(&self) -> &[u8] {
        self.view.as_ref().unwrap()
    }
}

// This is the view returned from the upload() method of the staging pool
// This allows us to write to the given buffer (it will submit this write when this gets dropped)
pub struct StagingViewWrite<'a> {
    view: Option<wgpu::BufferViewMut<'a>>,
    block: BlockId<'a>,
    buffer: &'a wgpu::Buffer,
    staging: &'a wgpu::Buffer,
    graphics: &'a Graphics,
    allocation: &'a Allocation,
    dst_offset: wgpu::BufferAddress,
    staging_offset: wgpu::BufferAddress,
    size: wgpu::BufferAddress,
}

impl<'a> AsMut<[u8]> for StagingViewWrite<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.view.as_mut().unwrap()
    }
}

impl<'a> AsRef<[u8]> for StagingViewWrite<'a> {
    fn as_ref(&self) -> &[u8] {
        self.view.as_ref().unwrap()
    }
}

impl<'a> Drop for StagingViewWrite<'a> {
    fn drop(&mut self) {
        drop(self.view.take().unwrap());
        self.staging.unmap();
        self.allocation.unmap_block(&self.block);

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
    fn map_block(
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

        // TODO: Actually do the buffer mapping here instead
        // of having it occur in the download / upload functions
        
        // Convert the range into a BlockId
        output.map(|(start, end)| {
            used.insert(last_index,(start, end));
            BlockId {
                buffer: &self.backing,
                start,
                end,
                allocation: self,
            }
        })
    }

    // Unmap a block of memory that was already mapped
    fn unmap_block(
        &self,
        block: &BlockId
    ) {
        let mut locked = self.used.lock();

        let index = locked.iter().cloned().position(|(start, end)| 
            start == block.start && end == block.end
        );

        if let Some(index) = index {
            locked.remove(index);
        }
    }
}

// Staging buffer belt that can re-use multiple mappable buffers for 
// multiple download / upload operations
// TODO: Re-write this to make it a global buffer allocater / manager
pub struct StagingPool {
    allocations: ConcVec<Allocation>,
}

impl StagingPool {
    // Create a new staging belt for upload / download
    pub fn new() -> Self {
        Self {
            allocations: ConcVec::new(),
        }
    }

    // Tries to find a free block of memory that we can use, and allocate a new one if needed
    fn find_or_allocate(
        &self,
        graphics: &Graphics,
        mode: wgpu::MapMode,
        capacity: wgpu::BufferAddress,
    ) -> BlockId {
        log::debug!("Looking for block with size {capacity} and mode {mode:?}...");

        // Iterates over each block and checks if any of them have enough space for "capacity"
        let block = self.allocations.iter().filter(|allocation| allocation.mode == mode).filter_map(|allocation| {
            // TODO: Implement proper concurrent block mapping
            if allocation.used.lock().is_empty() {
                allocation.map_block(capacity)
            } else {
                None
            }
        }).next();

        if let Some(block) = block {
            log::debug!("Found free block from buffer backed allocation");
            block
        } else {
            // Convert the map mode to the proper usages
            let usage = match mode {
                wgpu::MapMode::Read => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                wgpu::MapMode::Write => wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            };

            // Scale up the capacity (so we don't have to allocate a new block anytime soon)
            let overallocator = |capacity: u64| (capacity*4).next_power_of_two();

            // Allocate a new backing buffer as requested
            log::warn!("Did not find block, allocating new buffer with size {capacity} and mode {mode:?}");
            let backing = graphics.device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: overallocator(capacity),
                usage,
                mapped_at_creation: false,
            });

            // Create the allocation struct
            let allocation = Allocation {
                backing,
                size: capacity,
                mode,
                used: Mutex::new(Vec::new()),
            };
            
            // Add and fetch to make sure WE own the allocation
            let index = self.allocations.len();
            self.allocations.push(allocation);
            let allocation = self.allocations.get(index).unwrap();

            // Create a sub-block for the allocation
            allocation.map_block(capacity).unwrap()
        }
    }


    // Map a buffer for writing only (maps an intermediate staging buffer)
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn map_read<'a>(
        &'a self,
        buffer: &wgpu::Buffer,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingView<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            allocation,
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
                view: Some(view),
                allocation,
                block: BlockId {
                    allocation,
                    buffer: staging,
                    start,
                    end,
                },
                staging,
                graphics,
            })
        } else {
            None
        }
    }

    // Map a buffer for writing only (maps an intermediate staging buffer)
    // Src buffer must have the COPY_DST buffer usage flag
    pub fn map_write<'a>(
        &'a self,
        buffer: &'a wgpu::Buffer,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) -> Option<StagingViewWrite<'a>> {
        // Decomspoe the allocation buffer
        let BlockId {
            allocation,
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
                block: BlockId {
                    allocation,
                    buffer: staging,
                    start,
                    end,
                },
                view: Some(view),
                buffer,
                graphics,
                staging,
                dst_offset: offset,
                staging_offset: start,
                size,
                allocation,
            })
        } else {
            None
        }
    }

    // Request an immediate buffer  write
    // Src buffer must have the COPY_DST buffer usage flag
    pub fn write<'a>(
        &'a self,
        buffer: &wgpu::Buffer,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        src: &[u8],
    ) {
        // TODO: Optimize this shit
        let mut read = self.map_write(
            buffer,
            graphics,
            offset,
            src.len() as u64
        ).unwrap();
        read.as_mut().copy_from_slice(src);
    }

    // Request an immediate buffer read
    // Src buffer must have the COPY_SRC buffer usage flag
    pub fn read<'a>(
        &'a self,
        buffer: &wgpu::Buffer,
        graphics: &'a Graphics,
        offset: wgpu::BufferAddress,
        dst: &mut [u8]
    ) {
        // TODO: Optimize this shit
        let read = self.map_read(
            buffer,
            graphics,
            offset,
            dst.len() as u64
        ).unwrap();
        dst.copy_from_slice(read.as_ref());
    }
}
