use crate::{CommandBuffer, CommandPool, Device, Queue};
use ash::vk;
use std::time::{Duration, Instant};

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
    pub(crate) command_pool: &'a CommandPool,
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
}

impl<'a> Recorder<'a> {
    // Create a raw recorder using it's raw components
    pub(crate) unsafe fn from_raw_parts(
        command_buffer: &'a CommandBuffer,
        command_pool: &'a CommandPool,
        device: &'a Device,
        queue: &'a Queue,
    ) -> Self {
        Self {
            command_buffer,
            command_pool,
            device,
            queue,
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

    // Get the underlying queue that we will eventually submit to
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    // Submit the recorder immediatly and wait for the GPU to execute it
    pub unsafe fn immediate_submit(self) {
        self.queue.immediate_submit(self)
    }
}

// Synchronization
impl<'a> Recorder<'a> {
    // Full pipeline barrier
    pub unsafe fn cmd_full_pipeline_barrier(&mut self) {
        self.device().raw().cmd_pipeline_barrier(
            self.command_buffer().raw(),
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[],
        );
    }

    // Specific buffer memory barrier
    pub unsafe fn cmd_buffer_memory_barrier(
        &mut self,
        barrier: vk::BufferMemoryBarrier,
    ) {
        self.device().raw().cmd_pipeline_barrier(
            self.command_buffer().raw(),
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[barrier],
            &[],
        );
    }

    // Specific image memory barrier
    pub unsafe fn cmd_image_memory_barrier(
        &mut self,
        barrier: vk::ImageMemoryBarrier,
    ) {
        self.device().raw().cmd_pipeline_barrier(
            self.command_buffer().raw(),
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
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
        self.device().raw().cmd_bind_index_buffer(
            self.command_buffer().raw(),
            buffer,
            offset,
            index_type,
        );
    }

    // Bind vertex buffers to the command buffer render pass
    pub unsafe fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        self.device().raw().cmd_bind_vertex_buffers(
            self.command_buffer().raw(),
            first_binding,
            &buffers,
            &offsets,
        );
    }

    // Copy a buffer to another buffer in GPU memory
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        regions: &[vk::BufferCopy],
    ) {
        self.device().raw().cmd_copy_buffer(
            self.command_buffer().raw(),
            src,
            dst,
            &regions,
        );
    }

    // Copy an image to a buffer in GPU memory
    pub unsafe fn cmd_copy_image_to_buffer(
        &mut self,
        buffer: vk::Buffer,
        image: vk::Image,
        layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        self.device().raw().cmd_copy_image_to_buffer(
            self.command_buffer().raw(),
            image,
            layout,
            buffer,
            regions,
        );
    }

    // Clear a buffer to zero
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
        self.device().raw().cmd_fill_buffer(
            self.command_buffer().raw(),
            buffer,
            offset,
            size,
            0,
        );
    }

    // Update the buffer using memory that is directly stored within the command buffer
    pub unsafe fn cmd_update_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        data: &[u8],
    ) {
        self.device().raw().cmd_update_buffer(
            self.command_buffer().raw(),
            buffer,
            offset,
            data,
        );
    }
}

// Image commands
impl<'a> Recorder<'a> {
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
        self.device().raw().cmd_blit_image(
            self.command_buffer().raw(),
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions,
            filter,
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
        self.device().raw().cmd_clear_color_image(
            self.command_buffer().raw(),
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
        self.device().raw().cmd_copy_image(
            self.command_buffer().raw(),
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions,
        )
    }

    // Copy a buffer to an image in GPU memory
    pub unsafe fn cmd_copy_buffer_to_image(
        &mut self,
        src_buffer: vk::Buffer,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        self.device().raw().cmd_copy_buffer_to_image(
            self.command_buffer().raw(),
            src_buffer,
            dst_image,
            dst_image_layout,
            regions
        )
    }
}

// Render pass commands

impl<'a> Recorder<'a> {
    // Begin a render pass
    pub unsafe fn cmd_begin_render_pass(
        &mut self,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        image_views: &[vk::ImageView],
        rect: vek::Rect<i32, u32>,
    ) {
        let mut attachments = vk::RenderPassAttachmentBeginInfo::builder()
            .attachments(image_views);

        let begin_info = vk::RenderPassBeginInfo::builder()
            .framebuffer(framebuffer)
            .render_pass(render_pass)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D {
                    x: rect.x,
                    y: rect.y,
                },
                extent: vk::Extent2D {
                    width: rect.w,
                    height: rect.h,
                },
            })
            .push_next(&mut attachments);

        self.device().raw().cmd_begin_render_pass(
            self.command_buffer.raw(),
            &begin_info,
            vk::SubpassContents::INLINE
        );
    }
    
    // End the currently active render pass
    pub unsafe fn cmd_end_render_pass(&mut self) {
        self.device().raw().cmd_end_render_pass(self.command_buffer().raw());
    }
    
    // Bind a pipeline to the bind point
    pub unsafe fn cmd_bind_pipeline(&mut self, pipeline: vk::Pipeline, point: vk::PipelineBindPoint) {
        self.device().raw().cmd_bind_pipeline(self.command_buffer().raw(), point, pipeline);
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
    queue: &'a Queue,
}

impl<'a> Submission<'a> {
    // Create a submission (only used within queue)
    pub fn new(
        command_pool: &'a CommandPool,
        command_buffer: &'a CommandBuffer,
        device: &'a Device,
        queue: &'a Queue,
    ) -> Self {
        Self {
            flushed: false,
            command_pool,
            command_buffer,
            device,
            queue,
        }
    }

    // Wait until the submission completes, and return the elapsedtime
    pub fn wait(mut self) -> Duration {
        let i = Instant::now();
        self.flush_then_wait();
        let elapsed = i.elapsed();
        log::warn!(
            "Waited for {:?} for cmd buffer execution",
            elapsed
        );
        elapsed
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
