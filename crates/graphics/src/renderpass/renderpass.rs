use vulkan::vk;

use crate::{Graphics, ActiveRenderPass, DynamicAttachment};

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
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            let graphics = Graphics::global();
            graphics
                .device()
                .destroy_render_pass_and_framebuffer(self.render_pass, self.framebuffer);
        }
    }
}

impl RenderPass {
    // Create a new render pass with the given attachment layouts
    pub unsafe fn new(
        attachment_image_infos: &[vk::FramebufferAttachmentImageInfo],
        rect: vek::Rect<i32, u32>
    ) -> Self {
        // Create the framebuffer for this render pass
        let graphics = Graphics::global();
        let extent = graphics.swapchain().extent();
        let framebuffer = graphics.device().create_frame_buffer(
            attachment_image_infos,
            extent
        );

        // Create the actual render pass
        let render_pass = graphics.device().create_render_pass(
            &[],
            &[],
        );

        Self {
            render_pass,
            framebuffer,
            rect,
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
    pub unsafe fn begin(&mut self, attachments: &[&mut dyn DynamicAttachment]) -> ActiveRenderPass {
        let graphics = Graphics::global();
        let device = graphics.device();
        let mut recorder = graphics.queue().acquire(device);
        recorder.cmd_begin_render_pass(
            self.render_pass,
            self.framebuffer,
            todo!(),
            self.rect,
        );

        ActiveRenderPass {
            renderpass: todo!(),
            recorder: todo!(),
        }
    }
}
