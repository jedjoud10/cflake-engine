use crate::{Device, Queue};
use ash::vk;
use gpu_allocator::{vulkan::Allocation, MemoryLocation};
use parking_lot::Mutex;

// Sub buffer block that is accessible in the staging pool
pub struct StagingBlock {
    index: usize,
    buffer: vk::Buffer,
    data: *mut u8,
    start: u64,
    end: u64,
}

impl StagingBlock {
    // Get the underlying vulkan buffer
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    // Get the mapped data immutably
    pub fn mapped_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size()) }
    }

    // Get the mapped data mutably
    pub fn mapped_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data, self.size())
        }
    }

    // Get the offset of this block into the actual sub buffer
    pub fn offset(&self) -> u64 {
        self.start
    }

    // Get the size of this sub buffer block
    pub fn size(&self) -> usize {
        (self.end - self.start) as usize
    }

    // Get the buffer chunk ID
    pub fn chunk(&self) -> usize {
        self.index
    }
}

unsafe impl Sync for StagingBlock {}
unsafe impl Send for StagingBlock {}

// Sub buffer used by the staging pool
pub(crate) struct StagingBuffer {
    raw: vk::Buffer,
    index: usize,
    allocation: Allocation,
    size: u64,
    used: Vec<(u64, u64)>,
}

impl StagingBuffer {
    // Check if the subbuffer has a free unused block of memory that we can use
    // This will return None if it could not find a buffer with the appropriate size within this block
    fn find_free_block_and_lock(
        &mut self,
        size: u64,
    ) -> Option<StagingBlock> {
        // Keep track of empty spaces within the sub buffer
        let mut last = 0u64;
        let mut output = None;

        // Try to find a free block of memory within the used blocks
        for (start, end) in self.used.iter() {
            if *start != last {
                if (start - last) > size {
                    output = Some((last, *start));
                }
            }

            last = *end;
        }

        // Try to find a free block at the end of the used blocks of memory
        if let Some((_, end)) = self.used.last() {
            let potential_block_size = self.size - end;
            if output.is_none() && (potential_block_size > size) {
                output = Some((*end, *end + size));
            }
        }

        // Try to find a free block at the start of the used blocks of memory
        if self.used.is_empty() {
            if output.is_none() && (self.size >= size) {
                output = Some((0, size));
            }
        }

        // Convert the found block of memory into a SubBufferBlock
        output.map(|(start, end)| {
            let ptr = unsafe {
                (self.allocation.mapped_ptr().unwrap().as_ptr()
                    as *mut u8)
                    .add(start.try_into().unwrap())
            };
            self.used.push((start, end));

            StagingBlock {
                index: self.index,
                buffer: self.raw,
                data: ptr,
                start,
                end,
            }
        })
    }
}

// A staging pool is used to transfer data between GPU and CPU memory
// This is a round robin buffer of mappable host visible buffers
// Stolen from https://docs.rs/vulkano/latest/vulkano/buffer/cpu_pool/struct.CpuBufferPool.html
pub struct StagingPool {
    subbuffers: Mutex<Vec<StagingBuffer>>,
}

impl StagingPool {
    // Create a new staging pool that will be stored within the graphical context
    pub unsafe fn new() -> Self {
        Self {
            subbuffers: Mutex::new(Vec::new()),
        }
    }

    // Force the allocation of a new block in memory, even though we might have
    // free blocks that we can reuse
    unsafe fn allocate(
        &self,
        device: &Device,
        queue: &Queue,
        size: u64,
    ) -> StagingBlock {
        // Use a bigger capacity just so we don't have to allocate as many times
        let upper = size * 8;

        // Create the underlying staging buffer memory
        let used = vk::BufferUsageFlags::TRANSFER_SRC
            | vk::BufferUsageFlags::TRANSFER_DST;
        let (buffer, allocation) = device.create_buffer(
            upper,
            used,
            MemoryLocation::GpuToCpu,
            queue,
        );

        // Get the staging buffer index
        let mut lock = self.subbuffers.lock();
        let index = lock.len();

        // Initialize the staging buffer struct
        let mut buffer = StagingBuffer {
            raw: buffer,
            index,
            allocation,
            size: upper,
            used: Vec::new(),
        };
        let block = buffer.find_free_block_and_lock(size).unwrap();

        // Add the staging buffer locally
        lock.push(buffer);
        drop(lock);
        block
    }

    // Get a sub-buffer or create a new one if we don't have a free one for use
    // The given buffer has the flags TRANSFER_SRC and TRANSFER_DST only
    pub unsafe fn lock(
        &self,
        device: &Device,
        queue: &Queue,
        size: u64,
    ) -> StagingBlock {
        let mut lock = self.subbuffers.lock();
        let find = lock
            .iter_mut()
            .enumerate()
            .find_map(|(_, sub)| sub.find_free_block_and_lock(size));

        // TODO: Check if the given buffer range is not in use by the GPU

        // Allocate a new buffer if we can't find one
        if let None = find {
            drop(lock);
            log::warn!("Could not find subbuffer block of size {size}, allocating a new one...");
            self.allocate(device, queue, size)
        } else {
            find.unwrap()
        }
    }

    // Unlock a buffer and return it to the staging pool
    pub unsafe fn unlock(
        &self,
        device: &Device,
        block: StagingBlock,
    ) {
        // TODO:
        // Check if the buffer is still in use by the GPU
        // Unlock when it is not in use, and add a callback to unlock it
    }

    // Deallocate all the buffers and blocks that we internally allocated
    pub(super) unsafe fn deallocate(&self, device: &Device) {
        device.wait();
        log::debug!("Deallocating staging buffers and sub-blocks...");
        let mut locked = self.subbuffers.lock();
        let mut block_count = 0;
        let mut buffer_count = 0;
        for buffer in locked.drain(..) {
            device.destroy_buffer(buffer.raw, buffer.allocation);
            block_count += buffer.used.len();
            buffer_count += 1;
        }

        log::debug!("Deallocated {block_count} sub-blocks and {buffer_count} buffers")
    }
}
