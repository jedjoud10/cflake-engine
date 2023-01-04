use vulkan::vk;

use crate::{DynamicAttachment, Graphics};

// In vanilla vulkan, render passes and frame buffers are completely separate, but since we will be using
// This is a wrapper around a Vulkan render pass that will read/write from/to specific attachments
// VK_KHR_imagless_framebuffer we can store them together for simplicity. So this wrapper isn't really a
// 1: 1 wrapper around a Vulkan render pass, and it abstracts a bit more than that
// TODO: Handle safer destruction when this gets dropped
pub struct RenderPass {
    // Raw vulkan
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    rect: vek::Rect<i32, u32>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl Drop for RenderPass {
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

impl RenderPass {
    // Create a new render pass with the given attachment layouts
    pub unsafe fn new(
        graphics: &Graphics,
        format: vk::Format,
        attachment_image_infos: &[vk::FramebufferAttachmentImageInfo],
        rect: vek::Rect<i32, u32>,
    ) -> Self {
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
        let (render_pass, framebuffer) =
            graphics.device().create_render_pass_framebuffer(
                &attachment,
                &[*subpass],
                &[],
                attachment_image_infos,
                rect.extent(),
                1,
            );

        Self {
            render_pass,
            framebuffer,
            rect,
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

    // Begin the render pass by using the given render attachments
    /*
    pub unsafe fn begin<'a, 'r>(&'a mut self, attachments: &[vk::ImageView], graphics: &'r Graphics,) -> ActiveRenderPass<'a, 'r> {
        let device = graphics.device();
        let mut recorder = graphics.queue().acquire(device);
        recorder.cmd_begin_render_pass(
            self.render_pass,
            self.framebuffer,
            attachments,
            self.rect,
        );
        log::debug!("Begin render pass");

        ActiveRenderPass {
            renderpass: self,
            recorder,
        }
    }
    */
}
