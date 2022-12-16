use ash::vk;
use gpu_allocator::{vulkan::Allocation, MemoryLocation};
use parking_lot::Mutex;
use crate::{Device, Queue};

// Sub buffer block that is accessible in the staging pool
pub struct SubBufferBlock {
    index: usize,
    buffer: vk::Buffer,
    data: *mut u8,
    start: usize,
    end: usize,
}

impl SubBufferBlock {
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
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size()) }
    }

    // Get the size of this sub buffer block
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    // Get the buffer chunk ID
    pub fn chunk(&self) -> usize {
        self.index
    }
}

unsafe impl Sync for SubBufferBlock {}
unsafe impl Send for SubBufferBlock {}


// Sub buffer used by the staging pool
pub(crate) struct SubBuffer {
    raw: vk::Buffer,
    index: usize,
    allocation: Allocation,
    size: u64,
    used: Vec<(usize, usize)>,
}

impl SubBuffer {
    // Check if the subbuffer has a free unused block of memory that we can use
    fn find_free_block_and_lock(&mut self, staging: &StagingPool, size: u64) -> Option<SubBufferBlock> {
        let mut last = 0usize;
        let mut output = None;
        for (start, end) in self.used.iter() {
            if start != end {
                if ((start - last) as u64) > size {
                    output = Some((last, *start));
                }
            }
            
            last = *end;
        }

        let output = if let Some(x) = output {
            self.used.push(x);
            Some(x)
        } else {
            None
        };

        

        output.map(|(start, end)| {
            let ptr = unsafe {
                (self.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8)
                .add(start)
            };
        
            SubBufferBlock {
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
// This is a ring buffer of mappable device local buffers
// Stolen from https://docs.rs/vulkano/latest/vulkano/buffer/cpu_pool/struct.CpuBufferPool.html
pub(crate) struct StagingPool {
    subbuffers: Mutex<Vec<SubBuffer>>,
}

impl StagingPool {
    // Create a new staging pool that will be stored within the graphical context
    pub unsafe fn new() -> Self {
        log::debug!("Creating a new staging pool");
        Self {
            subbuffers: Mutex::new(Vec::new())
        }
    }
    
    // Force the allocation of a new block in memory, even though we might have
    // free blocks that we can reuse
    unsafe fn allocate(&self, device: &Device, queue: &Queue, size: u64) -> SubBufferBlock {
        let used = vk::BufferUsageFlags::TRANSFER_SRC |  vk::BufferUsageFlags::TRANSFER_DST;
        let (buffer, allocation) = device.create_buffer(size, used, MemoryLocation::GpuToCpu, queue);
        let mut lock = self.subbuffers.lock();
        let index = lock.len();
        let mut subbuffer = SubBuffer { raw: buffer, index, allocation, size, used: Vec::new() };
        let block = subbuffer.find_free_block_and_lock(self, size).unwrap();
        lock.push(subbuffer);
        drop(lock);
        block
    }

    // Get a sub-buffer or create a new one if we don't have a free one for use
    // The given buffer has the flags TRANSFER_SRC and TRANSFER_DST only
    pub unsafe fn lock(&self, device: &Device, queue: &Queue, size: u64) -> SubBufferBlock {
        let mut lock = self.subbuffers.lock();
        let find = lock
            .iter_mut()
            .enumerate()
            .find_map(|(i, sub)| {
                sub.find_free_block_and_lock(self, size)
            });

        // Check if the given buffer range is not in use by the GPU
        //todo!();

        // Allocate a new buffer if we can't find one
        if let None = find {
            drop(lock);
            self.allocate(device, queue, size)
        } else { find.unwrap() }
    }

    // Unlock a buffer and return it to the staging pool
    // Note: This might be called with a buffer that is still in use by the GPU,
    // in which case this command would basically act as if the sub buffer was still in use 
    pub unsafe fn unlock(&self, device: &Device, block: SubBufferBlock) {
        
    }
}