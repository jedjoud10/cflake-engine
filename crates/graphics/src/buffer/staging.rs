use parking_lot::Mutex;
use vulkan::{Device, vk, Queue};

// Sub buffer used by the staging pool
struct SubBuffer {
    raw: vk::Buffer,
    allocation: vulkan::Allocation,
    size: u64,
    used: Vec<(usize, usize)>,
}

impl SubBuffer {
    // Check if the subbuffer has a free unused block of memory that we can use
    fn find_free_block_and_lock(&mut self, size: u64) -> Option<(usize, usize)> {
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

        if let Some(x) = output {
            self.used.push(x);
            Some(x)
        } else {
            None
        }
    }
}


// A staging pool is used to transfer data between GPU and CPU memory
// This is a ring buffer of mappable device local buffers
// Stolen from https://docs.rs/vulkano/latest/vulkano/buffer/cpu_pool/struct.CpuBufferPool.html
pub struct StagingPool {
    subbuffers: Mutex<Vec<SubBuffer>>,
}

// Sub buffer block that is accessible in the staging pool
pub struct SubBufferBlock {
    index: usize,
    buffer: vk::Buffer,
    data: *mut u8,
    start: usize,
    end: usize,
}

unsafe impl Sync for SubBufferBlock {}
unsafe impl Send for SubBufferBlock {}

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

impl StagingPool {
    // Create a new staging pool that will be stored within the graphical context
    pub fn new() -> Self {
        Self {
            subbuffers: Mutex::new(Vec::new())
        }
    }
    
    // Force the allocation of a new block in memory, even though we might have
    // free blocks that we can reuse
    fn allocate(&self, device: &Device, queue: &Queue, size: u64) -> SubBufferBlock {
        todo!()
    }

    // Get a sub-buffer or create a new one if we don't have a free one for use
    // The given buffer has the flags TRANSFER_SRC and TRANSFER_DST only
    pub fn lock(&self, device: &Device, queue: &Queue, size: u64) -> SubBufferBlock {
        let mut lock = self.subbuffers.lock();
        let find = lock
            .iter_mut()
            .enumerate()
            .find_map(|(i, sub)| {
                sub.find_free_block_and_lock(size).map(|(start, end)| SubBufferBlock {
                    index: i,
                    buffer: sub.raw,
                    data: unsafe { (sub.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8).add(start) },
                    start,
                    end,
                })
            });

        // Check if the given buffer range is not in use by the GPU
        todo!();

        // Allocate a new buffer if we can't find one
        if let None = find {
            drop(lock);
            self.allocate(device, queue, size)
        } else { find.unwrap() }
    }

    // Unlock a buffer and return it to the staging pool
    // Note: This might be called with a buffer that is still in use by the GPU,
    // in which case this command would basically act as if the sub buffer was still in use 
    pub fn unlock(&self, device: &Device, block: SubBufferBlock) {
        todo!()
    }
}