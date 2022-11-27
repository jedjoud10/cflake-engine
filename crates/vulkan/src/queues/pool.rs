use std::marker::PhantomData;
use ash::vk;
use parking_lot::Mutex;
use crate::Device;
use super::family::Family;


// A single command pool abstraction
// We technically should have one pool per thread
pub struct Pool {
    // Handle to the parent queue
    pub(super) queue: vk::Queue,

    // Raw vulkan command pool
    pub(super) alloc: vk::CommandPool,
    
    // All the command buffers allocated in this command pool
    pub(super) buffers: Mutex<Vec<vk::CommandBuffer>>,
}

impl Pool {
    // Reset the pool and reset all of the command buffers
    pub unsafe fn reset(&self, device: &Device) {
        device.device.reset_command_pool(self.alloc, Default::default()).unwrap();
    }

    // Allocate N number of command pools for this pool
    pub unsafe fn allocate_command_buffers(&self, device: &Device, number: usize, secondary: bool) {
        let level = if secondary {
            vk::CommandBufferLevel::SECONDARY
        } else { 
            vk::CommandBufferLevel::PRIMARY
        };

        let mut vec = self.buffers.lock();        
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.alloc)
            .command_buffer_count(number as u32)
            .level(level);
        let new = device.device.allocate_command_buffers(&create_info).unwrap();
        vec.extend(new);
        log::debug!("Allocated {number} new commands buffers of type {:?} for pool {:?}", level, self.alloc);
    }

    // Aquire a free command buffer as a recorder
    pub unsafe fn aquire_recorder<'a>(&self, device: &Device, flags: vk::CommandBufferUsageFlags) -> vk::CommandBuffer {
        let buffer = self.buffers.lock()[0];
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(flags);
        device.device.begin_command_buffer(buffer, &begin_info).unwrap();    
        log::debug!("Begin recording command buffer {:?}", buffer); 
        buffer
    }

    // Submit multiple recorders command buffers to the pool for execution
    pub unsafe fn submit_recorder_from_iter(
        &self,
        device: &Device,
        command_buffers: &[vk::CommandBuffer],
        signal: &[vk::Semaphore],
        wait: &[vk::Semaphore],
        masks: &[vk::PipelineStageFlags],
        fence: vk::Fence,
    ) {
        for buffer in command_buffers.iter(  ) {
            log::debug!("Stop recording command buffer {:?}", buffer); 
            device.device.end_command_buffer(*buffer).unwrap();
        }
        let submit = vk::SubmitInfo::builder()
            .signal_semaphores(signal)
            .wait_semaphores(wait)
            //.wait_dst_stage_mask(masks)
            .command_buffers(command_buffers);

        log::debug!("Submitting {} command buffers to queue {:?}", command_buffers.len(), self.queue); 
        device.device.queue_submit(self.queue, &[*submit], fence).unwrap()
    }
}