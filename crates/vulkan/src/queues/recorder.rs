use ash::vk;
use crate::{Device, Pool};

// Saved states that allow use to combine multiple recorders implicitly
#[derive(Default, Clone, Copy)]
pub(crate) struct State {}

// A recorder is a command buffer that is currently recording commands
// Recorders will automatically put semaphores and fences when necessary
// This will keep track of the operations done on specific buffers and
// automatically put semaphores between operations that affect the same object
pub struct Recorder<'d, 'p> {
    // Raw command buffer and it's idnex
    pub(crate) cmd: vk::CommandBuffer,
    pub(crate) index: usize,

    // Data related to context
    pub(crate) state: State,
    pub(crate) device: &'d Device,
    pub(crate) pool: &'p Pool,
}

impl<'d, 'p> Drop for Recorder<'d, 'p> {
    fn drop(&mut self) {
        let taken = std::mem::take(&mut self.state);
        self.pool.update_recorder_state(self.index, taken);
    }
}

// Image commands
impl<'d, 'p> Recorder<'d, 'p> {
    // Copy an image to another image
    //pub unsafe fn copy_image()

    // Clear an image to a specific color
    pub unsafe fn clear_image(
        &self, 
        src: vk::Image,
        value: vk::ClearColorValue,
        range: &[vk::ImageSubresourceRange],
    ) {
        
    }
}

// Buffer commands
impl<'d, 'p> Recorder<'d, 'p> {
    // Copy buffer contents to another buffer's contents'
    pub unsafe fn copy_buffer(
        &self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: &[vk::BufferCopy],
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