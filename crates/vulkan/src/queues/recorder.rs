use ash::vk;

use crate::Device;

// A recorder is a command buffer that is currently recording commands
// Recorders will automatically put semaphores and fences when necessar

// This will keep track of the operations done on specific buffers and 
// automatically put semaphores between operations that affect the same object
pub struct Recorder<'a> {
    pub(crate) cmd: vk::CommandBuffer,
    pub(crate) device: &'a Device,
}


// Buffer commands
impl<'a> Recorder<'a> {
    // Copy buffer contents to another buffer's contents'
    pub unsafe fn copy_buffer(&self, src: vk::Buffer, dst: vk::Buffer, regions: &[vk::BufferCopy]) {
    }

    // Clear the buffer "src"
    pub unsafe fn clear_buffer(&self, src: vk::Buffer, offset: u32, size: u32) {
    }
}