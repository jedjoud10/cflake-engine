use super::State;
use ash::vk;

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder {
    // Data used internally by the abstraction layer
    pub(super) index: usize,
    pub(super) pool: usize,
    pub(super) state: State,
    pub(super) force: bool,

    // Raw command buffer
    pub(super) raw_command_buffer: vk::CommandBuffer,
    pub(super) raw_command_pool: vk::CommandPool,
}

impl Recorder {
    // Get the raw command buffer used by this recorder
    // Note: The command buffer isn't currently recording
    pub fn raw_command_buffer(&self) -> vk::CommandBuffer {
        self.raw_command_buffer
    }

    // Get the raw command pool that allocated the cmd buffer
    pub fn raw_command_pool(&self) -> vk::CommandPool {
        self.raw_command_pool
    }
}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission {
    pub(crate) index: usize,
}

impl Submission {
    // Check if the submission has completed
    pub fn has_completed(&self) -> bool {
        todo!()
    }
    
    // Wait until the submission completes
    pub fn wait(&self) {
        todo!()
    }
}