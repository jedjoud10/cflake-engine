use crate::{Adapter, Device, Instance, Recorder, Submission};
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
        // Get the present and graphics queue family
        let family = adapter
            .queue_family_properties
            .iter()
            .enumerate()
            .position(|(i, props)| {
                // Check if the queue family supports the flags
                let flags = props
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS);

                // If the queue we must fetch must support presenting, fetch the physical device properties
                let presenting = !adapter
                    .queue_family_surface_supported[i]
                    || adapter.queue_family_surface_supported[i];
                flags && presenting
            })
            .unwrap() as u32;

        // Get the queue from the device
        let queue = device.raw().get_device_queue(family, 0);
        log::debug!(
            "Created the default graphics-present queue successfully"
        );

        Self {
            qfi: family,
            properties: adapter.queue_family_properties
                [family as usize],
            pools: vec![Pool::new(device, family)],
            queue,
        }
    }

    // Find a queue that supports the specific flags
    pub(super) unsafe fn pick_queue_family(
        family_properties: &[vk::QueueFamilyProperties],
        adapter: &Adapter,
        supports_presenting: bool,
        flags: vk::QueueFlags,
    ) -> u32 {
        family_properties
            .iter()
            .enumerate()
            .position(|(i, props)| {
                // Check if the queue family supporsts the flags
                let flags = props.queue_flags.contains(flags);

                // If the queue we must fetch must support presenting, fetch the physical device properties
                let presenting = !supports_presenting
                    || adapter.queue_family_surface_supported[i];
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

    // Aquire a new free command recorder that we can use to record commands
    // This might return a command buffer that is already in the recording state*
    pub unsafe fn acquire(
        &self,
        device: &Device,
        force: bool,
    ) -> Recorder {
        // Get the current thread's command pool
        // Allocate new one if not
        let pool = &self.pools[0];

        // Get a free command buffer
        let (index, buffer, state) = pool.find_free_and_lock();

        // Create the recorder
        Recorder {
            force,
            index,
            pool: 0,
            state,
            raw_command_buffer: buffer,
            raw_command_pool: pool.pool,
        }
    }

    // Submit the command buffer (this doesn't actually submit it, it only steals it's state)
    // You can use the "force" parameter to force the submission of this command buffer
    pub unsafe fn submit(
        &self,
        device: &Device,
        recorder: Recorder,
    ) -> Submission {
        log::warn!("Submitting (locally storing) command recorder");
        log::debug!(
            "Currently stored commands: {}",
            recorder.state.commands.len()
        );

        let pool = &self.pools[0];
        let index = recorder.index;
        if recorder.force {
            pool.submit(
                self.queue,
                device,
                recorder.index,
                recorder.state,
            );
        } else {
            pool.unlock(recorder.index, recorder.state);
        }

        Submission { index }
    }

    // Destroy the queue and the command pools
    pub unsafe fn destroy(&self, device: &Device) {
        for pool in &self.pools {
            pool.destroy(device);
        }
    }
}
