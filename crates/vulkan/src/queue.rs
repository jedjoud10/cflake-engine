use crate::{Adapter, Device, Instance, Recorder, Submission, CommandBufferTags};
use super::{Pool};
use ash::vk;

// This will be the main queue that we will access and submit data into
// For now I only support a single queue cause I am a bit dumb
pub struct Queue {
    // Queue family index
    pub(super) qfi: u32,

    // Queue family properties
    pub(super) properties: vk::QueueFamilyProperties,

    // Command pools that we can use
    pub(super) pools: Vec<Pool>,

    // Main queue that we submit command buffers to
    pub(super) queue: vk::Queue,
}

impl Queue {
    // Create the queue families, queues, and default pools
    pub unsafe fn new(
        device: &Device,
        adapter: &Adapter,
    ) -> Self {
        // Let one queue family handle everything
        let family = Self::pick_queue_family(
            adapter,
            true,
            vk::QueueFlags::GRAPHICS
             | vk::QueueFlags::COMPUTE
             | vk::QueueFlags::TRANSFER
        );

        // Get the queue from the device
        let queue = device.raw().get_device_queue(family, 0);
        log::debug!(
            "Created the default graphics-present queue successfully"
        );

        Self {
            qfi: family,
            properties: adapter.families.queue_family_properties
                [family as usize],
            pools: vec![Pool::new(device, family)],
            queue,
        }
    }

    // Find a queue that supports the specific flags
    pub(super) unsafe fn pick_queue_family(
        adapter: &Adapter,
        supports_presenting: bool,
        flags: vk::QueueFlags,
    ) -> u32 {
        adapter.families.queue_family_properties
            .iter()
            .enumerate()
            .position(|(i, props)| {
                // Check if the queue family supporsts the flags
                let flags = props.queue_flags.contains(flags);

                // If the queue we must fetch must support presenting, fetch the physical device properties
                let presenting = !supports_presenting
                    || adapter.families.queue_family_surface_supported[i];
                flags && presenting
            })
            .unwrap() as u32
    }

    // Get the queue family index of this queue
    pub fn queue_family_index(&self) -> u32 {
        self.qfi
    }

    // Get the queue's index within it's family
    pub fn queue_index(&self) -> u32 {
        0
    }

    // Get the queue properties and it's supported modes
    pub fn flags(&self) -> vk::QueueFlags {
        self.properties.queue_flags
    }

    // Get the underlying raw queue
    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    // Aquire a new free command recorder that we can use to record commands
    // This might return a command buffer that is already in the recording state*
    pub unsafe fn acquire(
        &self,
        device: &Device,
    ) -> Recorder {
        let pool = &self.pools[0];
        let (index, buffer, state) = pool.find_free_and_lock();

        // Create the recorder
        Recorder {
            index,
            pool: 0,
            state,
            raw_command_buffer: buffer,
            raw_command_pool: pool.pool,
        }
    }

    // Submit the command buffer (this doesn't actually submit it, it only steals it's state)
    // You can use the "force" parameter to force the submission of this command buffer
    pub unsafe fn submit<'a>(
        &'a self,
        device: &'a Device,
        recorder: Recorder,
    ) -> Submission {
        log::debug!("Submitting (locally storing) command recorder");
        log::debug!(
            "Currently stored commands: {}",
            recorder.state.commands.len()
        );

        let pool = &self.pools[0];
        let index = recorder.index;
        let force = recorder.state.commands.len() > 8 && !recorder.state.commands.is_empty();
        if force {
            pool.submit(
                self.queue,
                device,
                recorder.index,
                recorder.state,
            );
        } else {
            if recorder.state.commands.is_empty() {
                log::warn!("The command buffer is empty");
            }
 
            pool.unlock(recorder.index, recorder.state);
        }

        Submission { queue: self.queue, index, pool, device, flushed: false }
    }

    // Flush unsubmitted command buffers to this queue
    pub unsafe fn flush(&self, device: &Device) {
        let pool = &self.pools[0];
        pool.flush_all(self.queue, device);
    }

    // Wait until all the data submitted to this queue finishes executing on the GPU
    pub fn wait(&self, device: &Device) {
        unsafe { device.raw().queue_wait_idle(self.queue()).unwrap() };
    }

    // Destroy the queue and the command pools
    pub unsafe fn destroy(&self, device: &Device) {
        for pool in &self.pools {
            pool.destroy(device);
        }
    }
}
