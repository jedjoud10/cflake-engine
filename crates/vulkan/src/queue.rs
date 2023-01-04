use super::CommandPool;
use crate::{Adapter, Device, Instance, Recorder, Submission};
use ash::vk;

// This will be the main queue that we will access and submit data into
// For now I only support a single queue cause I am a bit dumb
pub struct Queue {
    // Queue family index
    pub(super) qfi: u32,

    // Queue family properties
    pub(super) properties: vk::QueueFamilyProperties,

    // Command pools that we can use
    pub(super) pools: Vec<CommandPool>,

    // Main queue that we submit command buffers to
    pub(super) queue: vk::Queue,
}

impl Queue {
    // Create the queue families, queues, and default pools
    pub unsafe fn new(device: &Device, adapter: &Adapter) -> Self {
        // Let one queue family handle everything
        let family = Self::pick_queue_family(
            adapter,
            true,
            vk::QueueFlags::GRAPHICS
                | vk::QueueFlags::COMPUTE
                | vk::QueueFlags::TRANSFER,
        );

        // Get the queue from the device
        let queue = device.raw().get_device_queue(family, 0);
        log::debug!(
            "Created the default graphics-present queue successfully"
        );

        Self {
            qfi: family,
            properties: adapter
                .family_properties()
                .queue_family_properties[family as usize],
            pools: vec![CommandPool::new(device, family)],
            queue,
        }
    }

    // Find a queue that supports the specific flags
    pub fn pick_queue_family(
        adapter: &Adapter,
        supports_presenting: bool,
        flags: vk::QueueFlags,
    ) -> u32 {
        adapter
            .family_properties()
            .queue_family_properties
            .iter()
            .enumerate()
            .position(|(i, props)| {
                // Check if the queue family supporsts the flags
                let flags = props.queue_flags.contains(flags);

                // If the queue we must fetch must support presenting, fetch the physical device properties
                let presenting = !supports_presenting
                    || adapter
                        .family_properties()
                        .queue_family_surface_supported[i];
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
    pub fn raw(&self) -> vk::Queue {
        self.queue
    }

    // Aquire a new free command recorder that we can use to record commands
    // This might return a command buffer that is already in the recording state*
    pub unsafe fn acquire<'a>(
        &'a self,
        device: &'a Device,
    ) -> Recorder<'a> {
        let command_pool = &self.pools[0];
        unsafe {
            let command_buffer = command_pool.start_recording(device);
            Recorder::from_raw_parts(
                command_buffer,
                command_pool,
                device,
                self,
            )
        }
    }

    // Return the command recorder back to the queue so we can chain new commands onto it
    pub unsafe fn chain(&self, recorder: Recorder) {
        recorder.command_pool().chain(recorder.command_buffer())
    }

    /*
    // Submit the command buffer and start executing it's commands on the GPU
    pub unsafe fn submit<'a>(
        &'a self,
        recorder: Recorder<'a>,
    ) -> Submission<'a> {
        let pool = recorder.command_pool;
        log::warn!(
            "Submitting the command buffer of index {}",
            recorder.command_buffer.index()
        );
        unsafe {
            pool.stop_recording(
                recorder.device(),
                &recorder.command_buffer,
            );
            pool.submit(
                self.raw(),
                recorder.device(),
                &recorder.command_buffer,
            );
        }
        Submission::new(
            recorder.command_pool,
            recorder.command_buffer,
            recorder.device,
            self,
        )
    }
    */

    // Submit the command buffer and wait until the commands have finished executing
    pub unsafe fn immediate_submit<'a>(
        &'a self,
        recorder: Recorder<'a>,
    ) {
        let pool = recorder.command_pool;
        log::warn!(
            "Submitting the command buffer of index {}",
            recorder.command_buffer.index()
        );
        unsafe {
            pool.stop_recording(
                recorder.device(),
                &recorder.command_buffer,
            );
            pool.submit(
                self.raw(),
                recorder.device(),
                &recorder.command_buffer,
            );
        }
        let submission = Submission::new(
            recorder.command_pool,
            recorder.command_buffer,
            recorder.device,
            self,
        );
        submission.wait();
    }

    // Wait until all the data submitted to this queue finishes executing on the GPU
    pub fn wait(&self, device: &Device) {
        unsafe { device.raw().queue_wait_idle(self.raw()).unwrap() };
    }

    // Destroy the queue and the command pools
    pub unsafe fn destroy(&self, device: &Device) {
        for pool in &self.pools {
            pool.destroy(device);
        }
    }
}
