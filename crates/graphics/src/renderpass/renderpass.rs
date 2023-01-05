use std::marker::PhantomData;
use vulkan::{vk, Recorder};
use crate::{Graphics, ColorLayout, DepthStencilLayout, ColorAttachments, DepthStencilAttachment, RenderPassBeginError, RenderPassInitializationError, GraphicsPipeline, Viewport};

// In vanilla vulkan, render passes and frame buffers are completely separate, but since we will be using
// This is a wrapper around a Vulkan render pass that will read/write from/to specific attachments
// VK_KHR_imagless_framebuffer we can store them together for simplicity. So this wrapper isn't really a
// 1: 1 wrapper around a Vulkan render pass, and it abstracts a bit more than that
pub struct RenderPass<C: ColorLayout, DS: DepthStencilLayout> {
    // Raw vulkan
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,

    // Dimensions
    extent: vek::Extent2<u32>,

    // We don't acually store the types
    _phantom_color: PhantomData<C>,
    _phantom_depth_stencil: PhantomData<DS>,

    // Keep the graphics API alive
    graphics: Graphics,
}

// TODO: Handle safer destruction when this gets dropped
impl<C: ColorLayout, DS: DepthStencilLayout> Drop for RenderPass<C, DS> {
    fn drop(&mut self) {
        unsafe {
            self.graphics
                .device()
                .destroy_framebuffer(
                    self.framebuffer,
                );
            self.graphics
                .device()
                .destroy_render_pass(
                    self.render_pass
                );
        }
    }
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Create a new render pass with some predefined dimensions 
    // TODO: Use multiple attachments and multiple subpasses
    pub fn new(
        graphics: &Graphics,
        extent: vek::Extent2<u32>,
    ) -> Result<Self, RenderPassInitializationError> {
        let format = C::untyped_texels()[0].format;
        let view_format = [format];
        let attachment_image_info =
            vk::FramebufferAttachmentImageInfo::builder()
                .width(extent.w)
                .height(extent.h)
                .view_formats(&view_format)
                .layer_count(1)
                .usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::TRANSFER_DST,
            );
        let attachment_image_infos = [*attachment_image_info];
        

        // FIXME
        let attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        let attachment = [*attachment];

        // FIXME
        let attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let attachment_ref = [*attachment_ref];

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attachment_ref);

        // Create a render pass and a framebuffer
        let (render_pass, framebuffer) = unsafe {
            // Create a render pass first
            let render_pass = graphics.device().create_render_pass(
                &attachment,
                &[*subpass],
                &[],
            );

            // Then create the framebuffer
            let framebuffer = graphics.device().create_framebuffer(
                &attachment_image_infos,
                extent,
                1,
                render_pass
            );

            (render_pass, framebuffer)
        };
            

        Ok(Self {
            render_pass,
            framebuffer,
            _phantom_color: PhantomData,
            _phantom_depth_stencil: PhantomData,
            extent,
            graphics: graphics.clone(),
        })
    }

    // Get the underlying raw Vulkan render pass
    pub fn renderpass(&self) -> vk::RenderPass {
        self.render_pass
    }

    // Get the underlying raw Vulkan framebuffer
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
    }

    // Resize the render pass' framebuffer
    pub fn resize(&mut self, extent: vek::Extent2<u32>) {
        log::debug!("Resize render pass' framebuffer");
        
        self.extent = extent;
        let format = C::untyped_texels()[0].format;
        let view_format = [format];
        let attachment_image_info =
            vk::FramebufferAttachmentImageInfo::builder()
                .width(extent.w)
                .height(extent.h)
                .view_formats(&view_format)
                .layer_count(1)
                .usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::TRANSFER_DST,
            );
        let attachment_image_infos = [*attachment_image_info];

        unsafe {
            self.graphics.device().wait();
            let framebuffer = self.graphics.device().create_framebuffer(
                &attachment_image_infos,
                extent,
                1,
                self.render_pass
            );

            let old = std::mem::replace(&mut self.framebuffer, framebuffer);
            self.graphics.device().destroy_framebuffer(old);
            self.graphics.device().wait();
        } 
    }
}

pub struct Rasterizer<'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout> {
    viewport: Viewport,
    recorder: Recorder<'r>,
    _phantom_color_layout: PhantomData<&'c C>,
    _phantom_depth_stencil_layout: PhantomData<&'ds DS>,
}

impl<'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout> Rasterizer<'r, 'c, 'ds, C, DS> {
    pub fn bind_pipeline(
        &mut self,
        pipeline: &GraphicsPipeline,
    ) {
        unsafe {
            self.recorder.cmd_bind_pipeline(pipeline.raw(), vk::PipelineBindPoint::GRAPHICS);
        
            
            self.recorder.cmd_set_viewport(
                self.viewport.origin.x as f32,
                self.viewport.origin.y as f32,
                self.viewport.extent.w as f32,
                self.viewport.extent.h as f32,
                0.01,
                1.0
            );

            self.recorder.cmd_set_scissor(
                0,
                0,
                self.viewport.extent.w,
                self.viewport.extent.h
            );
            
        }
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32
    ) {
        unsafe {
            self.recorder.cmd_draw(vertex_count, instance_count, first_vertex, first_instance);
        }
    }


    pub fn end(mut self) -> Recorder<'r> {
        unsafe {
            self.recorder.cmd_end_render_pass();
            self.recorder
        }
    }
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Begin the render pass and return a rasterizer that we can use to draw onto the attachments
    // This will automatically resize the render pass if the attachments have been resized
    pub fn begin<'r, 'c, 'ds>(
        &'r mut self,
        color_attachments: impl ColorAttachments<'c, C>,
        depth_stencil_attachment: impl DepthStencilAttachment<'ds, DS>,
        _: Viewport,
    ) -> Result<Rasterizer<'r, 'c, 'ds, C, DS>, RenderPassBeginError> {
        let mut recorder = unsafe {
            self.graphics.queue().acquire(self.graphics.device())
        };


        let viewport = Viewport {
            origin: vek::Vec2::default(),
            extent: self.extent,
        };

        // FIXME
        let clear = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0; 4] },
        };
        let clear = [clear];

        unsafe {
            recorder.cmd_begin_render_pass(
                self.render_pass,
                self.framebuffer,
                &color_attachments.image_views(),
                &clear, 
                vek::Rect {
                    x: viewport.origin.x,
                    y: viewport.origin.y,
                    w: viewport.extent.w,
                    h: viewport.extent.h,
                }
            );
        }

        Ok(Rasterizer {
            viewport,
            recorder, _phantom_color_layout: PhantomData,
            _phantom_depth_stencil_layout: PhantomData
        })
    }
}