use crate::{Device, State};
use ash::vk;

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
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
    ) {
        let region = vk::BufferCopy {
            src_offset,
            dst_offset,
            size,
        };

        self.state.0.push(super::Command::BufferCopy {
            src,
            dst,
            size,
            regions: vec![region]
        });
    }

    // Clear the buffer "src"
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        src: vk::Buffer,
        offset: u32,
        size: u32,
    ) {
        self.state.0.push(super::Command::BufferFill {
            src,
            offset,
            size,
            data: 0,
        });
    }
}
