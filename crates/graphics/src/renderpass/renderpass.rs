use std::marker::PhantomData;
use vulkan::vk;
use crate::{Graphics, ColorLayout, DepthStencilLayout, ColorAttachments, DepthStencilAttachment};

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
                .destroy_render_pass_and_framebuffer(
                    self.render_pass,
                    self.framebuffer,
                );
        }
    }
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Create a new render pass with some predefined dimensions 
    // TODO: Utilize multiple attachments and multiple subpasses
    pub fn new(
        graphics: &Graphics,
        extent: vek::Extent2<u32>,
    ) -> Self {
        let format = todo!();
        let attachment_image_infos = todo!();

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
            graphics.device().create_render_pass_framebuffer(
                &attachment,
                &[*subpass],
                &[],
                attachment_image_infos,
                extent,
                1,
            )
        };
            

        Self {
            render_pass,
            framebuffer,
            _phantom_color: PhantomData,
            _phantom_depth_stencil: PhantomData,
            extent,
            graphics: graphics.clone(),
        }
    }

    // Get the underlying raw Vulkan render pass
    pub fn renderpass(&self) -> vk::RenderPass {
        self.render_pass
    }

    // Get the underlying raw Vulkan framebuffer
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Begin the render pass and return a rasterizer that we can use to draw onto the attachments
    pub fn begin<'c, 'ds>(
        &mut self,
        color_attachments: impl ColorAttachments<'c, C>,
        depth_stencil_attachment: impl DepthStencilAttachment<'ds, DS>,
    ) -> () {
    }
}