use std::marker::PhantomData;
use ash::vk;
use math::BitSet;
use parking_lot::Mutex;
use winit::platform::unix::x11::util::FrameExtents;
use crate::{Device, Recorder};
use super::family::Family;


// A single command pool abstraction
// We technically should have one pool per thread
pub struct Pool {
    // Handle to the parent queue
    pub(super) queue: vk::Queue,

    // Raw vulkan command pool
    pub(super) alloc: vk::CommandPool,
    
    // All the command buffers allocated in this command pool
    pub(super) buffers: Mutex<Vec<(vk::CommandBuffer, bool, usize)>>,

    // Fences that are signaled after we submit command buffers
    // First bool tells us if the fence is currently in use
    // SEcond bool tells us if the fence was signaled on the GPU
    pub(super) fences: Mutex<Vec<(vk::Fence, bool, bool)>>,
}

impl Pool {
    // Reset the pool and reset all of the command buffers
    pub unsafe fn reset(&self, device: &Device) {
        self.refresh_fence_signals(device);
        device.device.reset_command_pool(self.alloc, Default::default()).unwrap();
    }

    // Refresh the bitset states based on the fences and reset them
    pub unsafe fn refresh_fence_signals(&self, device: &Device) {
        log::debug!("Refresh fence signals locks aquired");
        let mut buffers = self.buffers.lock();
        let mut fences = self.fences.lock();
        
        // Indices of the command buffers
        let mut indices = Vec::<usize>::new();

        // Signaled fences
        let mut signaled = Vec::<vk::Fence>::new();

        // Update all the fences
        for (fence, fence_in_use, fence_cpu_signaled) in fences.iter_mut() {
            let fence_gpu_signaled = device.device.get_fence_status(*fence).unwrap();
            
            if *fence_in_use {
                *fence_cpu_signaled = fence_gpu_signaled;
            }
        }

        for (i, (_, using, index)) in buffers.iter().enumerate() {
            // Skip command buffers that are not submitted
            if *index == usize::MAX || !*using {
                continue;
            }

            // Check if the fence is signaled
            let (fence, fence_in_use, fence_cpu_signaled) = &mut fences[*index];

            // Keep track of free fences and their indices
            if *fence_cpu_signaled && *fence_in_use && !signaled.contains(fence) {
                signaled.push(*fence);
                indices.push(i);
                log::debug!("Fence {:?} was signaled, so we must reset it", fence)
            }
        }

        // Reset free fences
        if !signaled.is_empty() {
            log::debug!("Resetting {} fences", signaled.len());
            device.device.reset_fences(&signaled).unwrap();

            // Unsignal on the CPU side (correspodning reset_fences)
            for &index in indices.iter() {
                let (_, _, index) = buffers[index];
                let (_, fence_in_use, fence_cpu_signaled) = &mut fences[index];
                *fence_cpu_signaled = false;
                *fence_in_use = false;
            }
        }

        for free in indices {
            // Also reset the command buffer
            let (cmd, using, index) = &mut buffers[free];
            device.device.reset_command_buffer(*cmd, Default::default()).unwrap();
            log::debug!("Resetting command buffer {:?} that has fence index {index}", cmd);

            // Reset the fence index of this command buffer until we submit it again
            // This means that the command buffer can be used and it is reset
            *index = usize::MAX;
            *using = false;
        }
    }

    // Allocate N number of command pools for this pool
    pub unsafe fn allocate_command_buffers(pool: vk::CommandPool, buffers: &mut Vec<(vk::CommandBuffer, bool, usize)>, device: &Device, number: usize, secondary: bool) {
        let level = if secondary {
            vk::CommandBufferLevel::SECONDARY
        } else { 
            vk::CommandBufferLevel::PRIMARY
        };

        // Create the command buffers         
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .command_buffer_count(number as u32)
            .level(level);
        let new = device.device.allocate_command_buffers(&create_info).unwrap();

        // Combine command buffers and fence indices
        let new = new.into_iter().map(|cmd| {
            (cmd, false, usize::MAX)
        }).collect::<Vec<_>>();

        // Add the command buffers to our pool
        buffers.extend(new);
        log::debug!("Allocated {number} new commands buffers of type {:?} for pool {:?}", level, pool);
    }

    // Aquire a free command buffer as a recorder
    pub unsafe fn aquire_cmd_buffer(&self, device: &Device, flags: vk::CommandBufferUsageFlags) -> vk::CommandBuffer {
        self.refresh_fence_signals(device);
        let mut buffers = self.buffers.lock();

        // Try to find the index of a free command buffers
        let free = buffers.iter().position(|(_, using, fence)| 
            *fence == usize::MAX && !using
        );

        // Allocate new buffer if we don't have one available
        let cmd_index = if free.is_none() {
            log::warn!("Could not find free command buffer, allocating a new buffer");
            Self::allocate_command_buffers(self.alloc, &mut buffers, device, 1, false);
            buffers.len() - 1
        } else {
            log::debug!("Found free command buffer at index {}", free.unwrap());
            free.unwrap()
        };

        // Begin recording the command buffer
        let (buffer, old_using_state, _) = &mut buffers[cmd_index];
        *old_using_state = true;

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(flags);
        device.device.begin_command_buffer(*buffer, &begin_info).unwrap();    
        log::debug!("Begin recording command buffer {:?}", buffer); 
        *buffer
    }

    // Aquire an actual (free) recorder that we can use
    pub unsafe fn aquire_recorder<'a>(&self, device: &'a Device, flag: vk::CommandBufferUsageFlags, implicit: bool) -> Recorder<'a> {
        Recorder {
            cmd: self.aquire_cmd_buffer(device, flag),
            device,
        }
    }

    // Explcitly submit a recorder
    pub unsafe fn submit_recorder(
        &self,
        device: &Device,
        recorder: Recorder,
        signal: &[vk::Semaphore],
        wait: &[vk::Semaphore],
        masks: &[vk::PipelineStageFlags],
    ) -> vk::Fence {
        self.submit_cmd_buffers_from_iter(
            device,
            &[recorder.cmd],
            signal,
            wait,
            masks
        )
    }

    // Submit multiple recorders command buffers to the pool for execution
    pub unsafe fn submit_cmd_buffers_from_iter(
        &self,
        device: &Device,
        command_buffers: &[vk::CommandBuffer],
        signal: &[vk::Semaphore],
        wait: &[vk::Semaphore],
        masks: &[vk::PipelineStageFlags],
    ) -> vk::Fence {
        // Stop recording the command buffers
        for buffer in command_buffers.iter() {
            log::debug!("Stop recording command buffer {:?}", buffer); 
            device.device.end_command_buffer(*buffer).unwrap();
        }

        // Create the command buffers submit data
        let submit = vk::SubmitInfo::builder()
            .signal_semaphores(signal)
            .wait_semaphores(wait)
            //.wait_dst_stage_mask(masks)
            .command_buffers(command_buffers);

        // Find an unsignaled fence that we can use
        let (_, index) = self.find_free_fence(device);

        // Update the fence signaling on the CPU to tell it that
        // the fence is no longer free and it is unsignaled
        let mut lock = self.fences.lock();
        let fence = &mut lock[index];
        fence.1 = true;
        fence.2 = false;
        let fence = fence.0;

        // Go through each of the cached command buffers and update their indices
        let mut locked = self.buffers.lock();
        let iter = locked
            .iter_mut()
            .filter(|(cmd, _, _)| command_buffers.contains(cmd));
        for (_, using, old) in iter {
            *old = index;
            *using = true;
        }

        log::debug!("Submitting {} command buffers to queue {:?}", command_buffers.len(), self.queue); 
        device.device.queue_submit(self.queue, &[*submit], fence).unwrap();
        fence
    }

    // Find a free fence
    unsafe fn find_free_fence(&self, device: &Device) -> (vk::Fence, usize) {
        // Fetch a free fence that we can use
        self.refresh_fence_signals(device);
        log::debug!("Fence counts: {}. Looking for free fence...", self.fences.lock().len());
        let fence = self
            .fences
            .lock()
            .iter()
            .enumerate()
            .find(|(_, (fence, fence_in_use, fence_cpu_signaled))| {
                log::debug!("Fence {:?} has signaled state {} and usage state {}",
                    fence,
                    *fence_cpu_signaled,
                    *fence_in_use,
                );

                !*fence_in_use && !fence_cpu_signaled
            })
            .map(|(i, (fence, _, _))| (*fence, i));

        if let Some((_, i)) = fence {
            log::debug!("Found existing fence at index {i}");
        }

        // If we don't find a free fence, create a new one
        fence.unwrap_or_else(|| unsafe {
            log::warn!("Could not find a free fence, allocating a new one");
            let data = (device.create_fence(), false, false);
            log::debug!("Allocated new fence {:?}", data.0);
            let mut fences = self.fences.lock();
            fences.push(data);
            (data.0, fences.len()-1) 
        })
    }
}