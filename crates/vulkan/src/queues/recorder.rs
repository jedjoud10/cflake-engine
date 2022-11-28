use ash::vk;

use crate::{Device, Pool};

// A recorder is a command buffer that is currently recording commands
// Recorders will automatically put semaphores and fences when necessar

// This will keep track of the operations done on specific buffers and
// automatically put semaphores between operations that affect the same object
pub struct Recorder<'d, 'p> {
    pub(crate) cmd: vk::CommandBuffer,
    pub(crate) device: &'d Device,
    pub(crate) pool: &'p Pool,

    // If this is set, the recorder will implicitly
    // be submitted to the queue
    pub(crate) implicit: bool,
}

// Buffer commands
impl<'d, 'p> Recorder<'d, 'p> {
    // Copy buffer contents to another buffer's contents'
    pub unsafe fn copy_buffer(
        &self,
        _src: vk::Buffer,
        _dst: vk::Buffer,
        _regions: &[vk::BufferCopy],
    ) {
    }

    // Clear the buffer "src"
    pub unsafe fn clear_buffer(
        &self,
        _src: vk::Buffer,
        _offset: u32,
        _size: u32,
    ) {
    }
}

impl<'d, 'p> Drop for Recorder<'d, 'p> {
    fn drop(&mut self) {
        todo!()
    }
}
