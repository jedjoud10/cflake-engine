use crate::State;
use ash::vk;

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder {
    // Data used internally by the abstraction layer
    pub(crate) index: usize,
    pub(crate) pool: usize,
    pub(crate) state: State,
    pub(crate) force: bool,

    // Raw command buffer
    pub raw_command_buffer: vk::CommandBuffer,
    pub raw_command_pool: vk::CommandPool,
}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission {
    pub(crate) index: usize,
}
