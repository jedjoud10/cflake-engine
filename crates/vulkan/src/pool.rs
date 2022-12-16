use crate::Device;
use super::State;
use ash::vk;
use parking_lot::Mutex;

bitflags::bitflags! {
    pub(super) struct CommandBufferTags: u32 {
        // The command buffer is currently in use by the CPU
        const LOCKED = 1;

        // The command buffer is currently in use by the GPU
        const PENDING = 2;
    }
}

// Abstraction around a Vulkan command buffer
pub(super) struct CommandBuffer {
    // Underlying command buffer
    pub(super) raw: vk::CommandBuffer,

    // Fence for this command buffer
    pub(super) fence: vk::Fence,

    // State of the command buffer
    pub(super) state: Mutex<Option<State>>,

    // Tags that are applied to this command buffer
    pub(super) tags: Mutex<CommandBufferTags>,
}

// Abstraction around a Vulkan command pool
pub(super) struct Pool {
    // Underlying pool
    pub(super) pool: vk::CommandPool,

    // All the buffers that we allocated
    pub(super) buffers: Vec<CommandBuffer>,
}

impl Pool {
    // Create a new command pool and pre-allocate it
    pub(super) unsafe fn new(device: &Device, qfi: u32) -> Self {
        // Create the raw Vulkan command pool
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(qfi);
        let command_pool = device
            .raw()
            .create_command_pool(&pool_create_info, None)
            .unwrap();

        // Allocate some new command buffers
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(4)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let buffers = device
            .raw()
            .allocate_command_buffers(&allocate_info)
            .unwrap()
            .into_iter()
            .map(|raw| CommandBuffer {
                raw,
                state: Mutex::new(Some(State::default())),
                tags: Mutex::new(CommandBufferTags::empty()),
                fence: device.create_fence(),
            });

        Self {
            pool: command_pool,
            buffers: buffers.collect(),
        }
    }

    // Get a free command buffer from this pool and lock it for usage
    pub(super) unsafe fn find_free_and_lock(
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
        log::debug!("Found a free command buffer {:?}", buffer.raw);
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
        log::debug!("Unlocking buffer at index {index}");
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
        self.unlock(index, state);
        let state = self.buffers[index].state.lock().take().unwrap();
        self.record(device, buffer, state);

        let bufs = [buffer.raw];
        let info = vk::SubmitInfo::builder().command_buffers(&bufs);

        buffer.tags.lock().insert(CommandBufferTags::PENDING);

        log::debug!(
            "Submitting command buffer {:?} for execution",
            buffer.raw
        );
        device.raw().reset_fences(&[buffer.fence]).unwrap();
        device
            .raw()
            .queue_submit(queue, &[*info], buffer.fence)
            .unwrap();
        //device.raw().queue_wait_idle(queue).unwrap();
    }

    // Record a command buffer using it's given state
    pub(crate) unsafe fn record(
        &self,
        device: &Device,
        buffer: &CommandBuffer,
        state: State,
    ) {
        let converted = super::complete(state);
        device
            .raw()
            .begin_command_buffer(
                buffer.raw,
                &vk::CommandBufferBeginInfo::default(),
            )
            .unwrap();
        converted.insert(device.raw(), buffer.raw);
        device.raw().end_command_buffer(buffer.raw).unwrap();
    }

    // Flush unsubmitted command buffers to the given queue
    pub(crate) unsafe fn flush_all(
        &self,
        queue: vk::Queue,
        device: &Device,
    ) {
        let mut should_flush = Vec::<usize>::new();
        log::debug!("Explicit call to flush queue");
        for (index, buffer) in self.buffers.iter().enumerate() {
            let state = buffer.state.lock();
            if let Some(state) = &*state {
                if !state.commands.is_empty() {
                    should_flush.push(index);
                }
            }
        }

        log::debug!("Manually flushing {} cmd buffers", should_flush.len());
        for index in should_flush {
            let state = self.buffers[index].state.lock().take().unwrap();
            self.submit(queue, device, index, state);
        }
    }

    // Flush a specific command buffer (no-op if it was already flushed)
    // Flushing will tell the GPU to start executing the commands for this recorder
    pub(crate) unsafe fn flush_specific(
        &self,
        queue: vk::Queue,
        device: &Device,
        index: usize,
        fence: bool,
    ) -> Option<vk::Fence> {
        let buffer = &self.buffers[index];
        let mut lock = buffer.state.lock();
        let mut should_flush = false;
        if let Some(state) = &*lock {
            if !state.commands.is_empty() {
                should_flush = true;
            }
        }

        if should_flush {
            let state = lock.take().unwrap();
            drop(lock);
            self.submit(queue, device, index, state);
            return fence.then(|| buffer.fence);
        } else {
            return None;
        }
    }

    // Destroy the command pool
    pub(super) unsafe fn destroy(&self, device: &Device) {
        device.raw().device_wait_idle().unwrap();

        for wrapper in self.buffers.iter() {
            device.raw().destroy_fence(wrapper.fence, None);
            device.raw().free_command_buffers(self.pool, &[wrapper.raw]);
        }
        
        device.raw().destroy_command_pool(self.pool, None);
    } 
}
