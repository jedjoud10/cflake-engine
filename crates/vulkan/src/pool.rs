use crate::Device;
use ash::vk;
use parking_lot::Mutex;

// Abstraction around a Vulkan command buffer
pub struct CommandBuffer {
    // Raw vulkan sheize
    index: usize,
    raw: vk::CommandBuffer,
    fence: vk::Fence,

    // State tracking for this buffer
    free: Mutex<bool>,
    recording: Mutex<bool>,
}

impl CommandBuffer {
    // Get the index of the command buffer
    pub fn index(&self) -> usize {
        self.index
    }

    // Get the raw Vulkan command buffer
    pub fn raw(&self) -> vk::CommandBuffer {
        self.raw
    }

    // Get the command buffer's fence
    pub fn fence(&self) -> vk::Fence {
        self.fence
    }

    // Check if the command buffer is free
    pub fn is_free(&self) -> bool {
        *self.free.lock()
    }

    // Check if the command buffer is recording
    pub fn is_recording(&self) -> bool {
        *self.recording.lock()
    }
}

// Abstraction around a Vulkan command pool
pub struct CommandPool {
    pool: vk::CommandPool,
    buffers: Vec<CommandBuffer>,
}

impl CommandPool {
    // Create a new command pool and pre-allocate it
    pub(super) unsafe fn new(device: &Device, qfi: u32) -> Self {
        let pool = create_command_pool(qfi, device);
        let buffers = allocate_command_buffers(
            pool,
            device,
            4,
            vk::CommandBufferLevel::PRIMARY,
        );

        Self { pool, buffers }
    }

    // Get the internally used command buffers
    pub unsafe fn command_buffers(&self) -> &[CommandBuffer] {
        &self.buffers
    }

    // Gett he raw command pool
    pub unsafe fn raw(&self) -> vk::CommandPool {
        self.pool
    }

    // Try to find a free command buffer and begin recording it
    pub unsafe fn start_recording(
        &self,
        device: &Device,
    ) -> &CommandBuffer {
        let cmd =
            self.buffers.iter().find(|cmd| cmd.is_free()).unwrap();
        log::debug!("{}", cmd.index());
        *cmd.free.lock() = false;

        if !cmd.is_recording() {
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
            device
                .raw()
                .begin_command_buffer(cmd.raw, &begin_info)
                .unwrap();
            *cmd.recording.lock() = true;
        }

        cmd
    }

    // Stop recording a specific command buffer
    pub unsafe fn stop_recording(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
    ) {
        *command_buffer.free.lock() = false;
        *command_buffer.recording.lock() = false;
        device
            .raw()
            .end_command_buffer(command_buffer.raw())
            .unwrap();
    }

    // Submit a recorder to the queue
    pub unsafe fn submit(
        &self,
        queue: vk::Queue,
        device: &Device,
        buffer: &CommandBuffer,
    ) {
        let buffers = [buffer.raw()];
        let submit_info =
            vk::SubmitInfo::builder().command_buffers(&buffers);
        let submit_infos = [*submit_info];

        device
            .raw()
            .queue_submit(queue, &submit_infos, vk::Fence::null())
            .unwrap();
    }

    // Complete the lifetime of a specific command buffer
    pub unsafe fn complete(&self, buffer: &CommandBuffer) {
        *buffer.recording.lock() = false;
        *buffer.free.lock() = true;
    }

    /*
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
    */

    // Destroy the command pool
    pub(super) unsafe fn destroy(&self, device: &Device) {
        device.raw().device_wait_idle().unwrap();

        for wrapper in self.buffers.iter() {
            device.raw().destroy_fence(wrapper.fence, None);
            device
                .raw()
                .free_command_buffers(self.pool, &[wrapper.raw]);
        }

        device.raw().destroy_command_pool(self.pool, None);
    }
}

unsafe fn allocate_command_buffers(
    command_pool: vk::CommandPool,
    device: &Device,
    count: u32,
    level: vk::CommandBufferLevel,
) -> Vec<CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(count)
        .command_pool(command_pool)
        .level(level);
    let buffers = device
        .raw()
        .allocate_command_buffers(&allocate_info)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, raw)| CommandBuffer {
            index: i,
            raw,
            fence: device.create_fence(),
            free: Mutex::new(true),
            recording: Mutex::new(false),
        });
    buffers.collect()
}

unsafe fn create_command_pool(
    qfi: u32,
    device: &Device,
) -> vk::CommandPool {
    let pool_create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(qfi);
    let command_pool = device
        .raw()
        .create_command_pool(&pool_create_info, None)
        .unwrap();
    command_pool
}
