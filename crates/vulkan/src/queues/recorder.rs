use ash::vk;
use crate::{Device};

// Saved states that allow use to combine multiple recorders implicitly
#[derive(Default, Clone, Copy)]
pub(crate) struct State {}

// A recorder is a command buffer that is currently recording commands
// Recorders will automatically put semaphores and fences when necessary
// This will keep track of the operations done on specific buffers and
// automatically put semaphores between operations that affect the same object
pub struct Recorder<'d> {
    // Raw command buffer and it's idnex
    pub(crate) cmd: vk::CommandBuffer,
    pub(crate) index: usize,

    // Data related to context
    pub(crate) state: State,
    pub(crate) device: &'d Device,
}

// Buffer commands
impl<'d> Recorder<'d> {
    // Copy a buffer region to another buffer
    pub unsafe fn copy_buffer(
        &self,
        src: vk::Buffer,
        dst: vk::Buffer,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
    ) {
    }

    // Clear the buffer "src"
    pub unsafe fn clear_buffer(
        &self,
        src: vk::Buffer,
        offset: u32,
        size: u32,
    ) {
    }
}