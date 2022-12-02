use crate::{Pool, Queue, Device, Recorder};
use ash::vk;
use smallvec::SmallVec;

// Virtual queue family to dissacociate from actual queue family
// Virtual simply means that it creates two family structs for the same queue family index
pub struct Family {
    pub(crate) qfi: u32,
    pub(crate) properties: vk::QueueFamilyProperties,
    pub(crate) present: bool,
    pub(crate) allocate: bool,
    pub(crate) pools: Vec<Pool>,
    pub(crate) queue: Queue,
}

impl Family {
    // Create a new family with a specific number of init command pools
    pub(super) unsafe fn new(
        device: &Device,
        qfi: u32,
        properties: vk::QueueFamilyProperties,
        present: bool,
        allocate: bool,
    ) -> Self {
        Self {
            qfi,
            properties,
            present,
            allocate,
            pools: vec![Pool::new(device, qfi,  )],
            queue: Queue::new(qfi, device),
        }
    }

    // Aquire a free command pool for the current thread
    pub(super) unsafe fn aquire_pool(
        &self,
        allocate: Option<vk::CommandPoolCreateFlags>,
    ) -> &Pool {
    }

    // Aquire a free command buffer for the current thread
    pub(super) unsafe fn aquire_cmd_buffer(
        &self,
        pool: vk::CommandPoolCreateFlags,
        begin: vk::CommandBufferUsageFlags,
        level: vk::CommandBufferLevel,
        allocate: bool,
    ) -> vk::CommandBuffer {
        todo!()
    }

    // Submit multiples recorders to this queue
    // This will actually submit the recorders for execution
    pub unsafe fn submit<'d>(
        &self,
        device: &Device,
        signal_semaphores: &[vk::Semaphore],
        wait_semaphores: &[vk::Semaphore],
        wait_dst_stage_mask: &[vk::PipelineStageFlags],
        recorders: impl Iterator<Item = Recorder<'d>>,
        fence: Option<vk::Fence>,
    ) {
        self.queue.submit(
            device,
            signal_semaphores,
            wait_semaphores,
            wait_dst_stage_mask,
            recorders,
            fence
        );
    }

    // Automatically detect all the recorders that finished recording (partiallly)
    // and submit them to the GPU for completion
    pub(super) unsafe fn poll(
        &self,
        device: &Device,
    ) {
        for pool in &self.pools {
            
        }
    }
}
