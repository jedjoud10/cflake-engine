use std::time::{Duration, Instant};
use crate::{Device, CommandPool, CommandBuffer, Queue};
use ash::vk;

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
    pub(crate) command_pool: &'a CommandPool,
    pub(crate) device: &'a Device,
}

impl<'a> Recorder<'a> {
    // Create a raw recorder using it's raw components 
    pub(crate) unsafe fn from_raw_parts(
        command_buffer: &'a CommandBuffer,
        command_pool: &'a CommandPool,
        device: &'a Device,
    ) -> Self {
        Self {
            command_buffer,
            command_pool,
            device,
        }
    }

    // Get the command buffer from the recorder
    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    // Get the command pool from the recorder
    pub fn command_pool(&self) -> &CommandPool {
        &self.command_pool
    }

    // Get the underlying device from the device  
    pub fn device(&self) -> &Device {
        &self.device
    }
}


// Buffer commands
impl<'a> Recorder<'a> {
    // Bind an index buffer to the command buffer render pass
    pub unsafe fn cmd_bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: Vec<vk::Buffer>,
        offsets: Vec<vk::DeviceSize>,
    ) {
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: Vec<vk::BufferCopy>,
    ) {
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn cmd_copy_image_to_buffer(
        &mut self,
        buffer: vk::Buffer,
        image: vk::Image,
        layout: vk::ImageLayout,
        regions: Vec<vk::BufferImageCopy>,
    ) {
    }

    // Clear a buffer to zero
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        data: Vec<u8>,
    ) {
    }
}

// Image commands
impl<'a> Recorder<'a> {
    // Blit an image to another image in GPU memory
    pub unsafe fn cmd_blit_image(
        &mut self,
        src_image: vk::Image,
        src_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_layout: vk::ImageLayout,
        regions: Vec<vk::ImageBlit>,
        filter: vk::Filter,
    ) {
    }

    // Clear an image to a specific color 
    pub unsafe fn cmd_clear_image(
        &mut self,
        image: vk::Image,
        layout: vk::ImageLayout,
        color: vk::ClearColorValue,
        regions: Vec<vk::ImageSubresourceRange>,
    ) {
    }

    // Copy an image to another image in GPU memory
    pub unsafe fn cmd_copy_image(
        &mut self,
        src_image: vk::Image,
        src_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_layout: vk::ImageLayout,
        regions: Vec<vk::ImageCopy>,
    ) {
    }
}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission<'a> {
    flushed: bool,
    force: bool,
    
    // Vulkan wrappers
    command_pool: &'a CommandPool,
    command_buffer: &'a CommandBuffer,
    device: &'a Device,
    queue: &'a Queue,
}

impl<'a> Submission<'a> {   
    // Create a submission (only used within queue)
    pub fn new(command_pool: &'a CommandPool, command_buffer: &'a CommandBuffer, device: &'a Device, queue: &'a Queue) -> Self {
        Self {
            flushed: false,
            force: true,
            command_pool,
            command_buffer,
            device,
            queue
        }
    }

    // Wait until the submission completes, and return the elapsedtime
    pub fn wait(mut self) -> Duration {     
        let i = Instant::now();
        self.flush();   
        i.elapsed()
    }
    
    // Force an immediate flush of the buffer
    pub fn flush(&mut self) {
        if self.flushed {
            return;
        }

        // Flush the submission and start executing it on the GPU
        log::debug!("Waiting for submission {} from queue {:?}", self.command_buffer.index(), self.queue.raw());
        //let fence = unsafe { self.pool.flush_specific(self.queue, self.device, self.index, true) };
        /*
        log::debug!("Waiting on fence {:?}...", fence);
        
        // Wait for the fence (if we have one) to complete
        if let Some(fence) = fence {
            unsafe { self.device.raw().wait_for_fences(&[fence], true, u64::MAX).unwrap() };
        } else {
            log::warn!("Waiting on submission that doesn't have a fence!");
        }
        self.flushed = true;
        */
        self.device.wait();
        self.flushed = true;
        unsafe { self.command_pool.complete(self.command_buffer); }
    }
}

impl<'a> Drop for Submission<'a> {
    fn drop(&mut self) {
        self.flush();
    }
}