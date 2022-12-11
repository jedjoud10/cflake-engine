use crate::{Device, InsertVkCommand, State};
use ash::vk;
use parking_lot::Mutex;

bitflags::bitflags! {
    pub(crate) struct CommandBufferTags: u32 {
        // The command buffer is currently in use by the CPU
        const LOCKED = 1;

        // The command buffer is currently in use by the GPU
        const PENDING = 2;
    }
}

// Abstraction around a Vulkan command buffer
pub(crate) struct CommandBuffer {
    // Underlying command buffer
    pub(crate) raw: vk::CommandBuffer,

    // State of the command buffer
    pub(crate) state: Mutex<Option<State>>,

    // Tags that are applied to this command buffer
    pub(crate) tags: Mutex<CommandBufferTags>,
}

// Abstraction around a Vulkan command pool
pub(crate) struct Pool {
    // Underlying pool
    pub(crate) pool: vk::CommandPool,

    // All the buffers that we allocated
    pub(crate) buffers: Vec<CommandBuffer>,
}

impl Pool {
    // Create a new command pool and pre-allocate it
    pub(crate) unsafe fn new(device: &Device, qfi: u32) -> Self {
        // Create the raw Vulkan command pool
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(qfi);
        let command_pool = device
            .device
            .create_command_pool(&pool_create_info, None)
            .unwrap();

        // Allocate some new command buffers
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let buffers = device
            .device
            .allocate_command_buffers(&allocate_info)
            .unwrap()
            .into_iter()
            .map(|raw| CommandBuffer {
                raw,
                state: Mutex::new(Some(State::default())),
                tags: Mutex::new(CommandBufferTags::empty()),
            });

        Self {
            pool: command_pool,
            buffers: buffers.collect(),
        }
    }

    // Get a free command buffer from this pool and lock it for usage
    pub(crate) unsafe fn find_free_and_lock(
        &self,
    ) -> (usize, vk::CommandBuffer, State) {
        let index = self
            .buffers
            .iter()
            .position(|cmd_buffer| {
                let CommandBuffer { tags, .. } = cmd_buffer;
                let tags = tags.lock();

                // Check if the command buffer isn't in use on the CPU
                let locked =
                    !tags.contains(CommandBufferTags::LOCKED);

                // Check if the command buffer isn't in use on the GPU
                let pending =
                    !tags.contains(CommandBufferTags::PENDING);

                pending && locked
            })
            .unwrap();

        let buffer = &self.buffers[index];
        log::warn!("Found a free command buffer {:?}", buffer.raw);
        let mut tags = buffer.tags.lock();
        tags.insert(CommandBufferTags::LOCKED);
        let state = buffer.state.lock().take().unwrap();
        log::debug!(
            "Currently chained commands: {}",
            state.commands.len()
        );

        (index, buffer.raw, state)
    }

    // Store the state of a command buffer back into the pool
    pub(crate) unsafe fn unlock(&self, index: usize, state: State) {
        log::warn!("Unlocking buffer at index {index}");
        let buffer = &self.buffers[index];
        *buffer.state.lock() = Some(state);
        let mut tags = buffer.tags.lock();
        tags.remove(CommandBufferTags::LOCKED);
    }

    // Actually submit a command buffer for execution
    pub(crate) unsafe fn submit(
        &self,
        queue: vk::Queue,
        device: &Device,
        index: usize,
        state: State,
    ) {
        let buffer = &self.buffers[index];
        self.record(device, buffer, state);

        let bufs = [buffer.raw];
        let info = vk::SubmitInfo::builder().command_buffers(&bufs);

        buffer.tags.lock().insert(CommandBufferTags::PENDING);

        log::warn!(
            "Submitting command buffer {:?} for execution",
            buffer.raw
        );
        device
            .device
            .queue_submit(queue, &[*info], vk::Fence::null())
            .unwrap();
        device.device.queue_wait_idle(queue).unwrap();
    }

    // Record a command buffer using it's given state
    pub(crate) unsafe fn record(
        &self,
        device: &Device,
        buffer: &CommandBuffer,
        state: State,
    ) {
        let converted = crate::complete(state);
        device
            .device
            .begin_command_buffer(
                buffer.raw,
                &vk::CommandBufferBeginInfo::default(),
            )
            .unwrap();
        for group in converted {
            for command in group.commands {
                command.insert(&device.device, buffer.raw);
            }

            if let Some(barrier) = group.barrier {
                barrier.insert(&device.device, buffer.raw);
            }
        }
        device.device.end_command_buffer(buffer.raw).unwrap();
    }

    // Destroy the command pool
    pub(super) unsafe fn destroy(&self, device: &Device) {
        device.device.device_wait_idle().unwrap();
        device.device.destroy_command_pool(self.pool, None);
    } 
}
