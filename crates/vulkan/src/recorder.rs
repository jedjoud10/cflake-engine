use crate::{CommandBuffer, CommandPool, Device, Queue};
use ash::vk;
use std::time::{Duration, Instant};

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder {
    cmd: vk::CommandBuffer,
    device: ash::Device,
    index: usize,
}

impl Recorder {
    // Create a raw recorder using it's raw components
    pub(crate) unsafe fn from_raw_parts(
        command_buffer: &CommandBuffer,
    ) -> Self {
        todo!()
    }
}

// Synchronization
impl Recorder {
    // Full barrier
    pub unsafe fn cmd_full_barrier(&mut self) {
        self.device.cmd_pipeline_barrier(
            self.cmd,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[], &[], &[]);
    } 
}

// Buffer commands
impl Recorder {
    // Bind an index buffer to the command buffer render pass
    pub unsafe fn cmd_bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        self.device.cmd_bind_index_buffer(self.cmd, buffer, offset, index_type);
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        self.device.cmd_bind_vertex_buffers(self.cmd, first_binding, &buffers, &offsets);
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: &[vk::BufferCopy],
    ) {
        self.device.cmd_copy_buffer(self.cmd, src, dst, &regions);
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn cmd_copy_image_to_buffer(
        &mut self,
        buffer: vk::Buffer,
        image: vk::Image,
        layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        self.device.cmd_copy_image_to_buffer(
            self.cmd,
            image, layout, buffer,
            regions);
    }

    // Clear a buffer to zero
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
        self.device.cmd_fill_buffer(
            self.cmd,
            buffer, offset, size, 0);
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        data: &[u8],
    ) {
        self.device.cmd_update_buffer(
            self.cmd,
            buffer, offset, data);
    }
}

// Image commands
impl Recorder {
    // Blit an image to another image in GPU memory
    pub unsafe fn cmd_blit_image(
        &mut self,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::ImageBlit],
        filter: vk::Filter,
    ) {
        self.device.cmd_blit_image(
            self.cmd,
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions,
            filter
        )
    }

    // Clear an image to a specific color
    pub unsafe fn cmd_clear_image(
        &mut self,
        image: vk::Image,
        layout: vk::ImageLayout,
        color: vk::ClearColorValue,
        regions: &[vk::ImageSubresourceRange],
    ) {
        self.device.cmd_clear_color_image(
            self.cmd,
            image,
            layout,
            &color,
            regions,
        )
    }

    // Copy an image to another image in GPU memory
    pub unsafe fn cmd_copy_image(
        &mut self,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::ImageCopy],
    ) {
        self.device.cmd_copy_image(
            self.cmd,
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions
        )
    }
}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission<'a> {
    flushed: bool,

    // Vulkan wrappers
    command_pool: &'a CommandPool,
    command_buffer: &'a CommandBuffer,
    device: &'a Device,
}

impl<'a> Submission<'a> {
    // Create a submission (only used within queue)
    pub fn new(
        command_pool: &'a CommandPool,
        command_buffer: &'a CommandBuffer,
        device: &'a Device,
    ) -> Self {
        Self {
            flushed: false,
            command_pool,
            command_buffer,
            device,
        }
    }

    // Wait until the submission completes, and return the elapsedtime
    pub fn wait(mut self) -> Duration {
        let i = Instant::now();
        self.flush_then_wait();
        i.elapsed()
    }

    // Force an immediate flush of the buffer, and wait for it to complete
    pub fn flush_then_wait(&mut self) {
        if self.flushed {
            return;
        }
        
        // Wait for the command buffer to complete
        unsafe {
            self.command_pool.wait(self.device, self.command_buffer);
        }
        self.flushed = true;
    }
}

impl<'a> Drop for Submission<'a> {
    fn drop(&mut self) {
        self.flush_then_wait();
    }
}
