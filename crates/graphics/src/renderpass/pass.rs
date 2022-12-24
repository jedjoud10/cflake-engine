use vulkan::vk;

use crate::Graphics;


// In vanilla vulkan, render passes and frame buffers are completely separate, but since we will be using
// This is a wrapper around a Vulkan render pass that will read/write from/to specific attachments
// VK_KHR_imagless_framebuffer we can store them together for simplicity. So this wrapper isn't really a 
// 1: 1 wrapper around a Vulkan render pass, and it abstracts a bit more than that
pub struct RenderPass {
    // Raw vulkan
    renderpass: vk::RenderPass,
    framebuffer: vk::Framebuffer,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_render_pass(self.renderpass);
            self.graphics.device().destroy_framebuffer(self.framebuffer);
        }
    }
}

impl RenderPass {
    // Create a new render pass with the given attachments
    pub unsafe fn new(
        graphics: &Graphics,
    ) -> Self {
        todo!()
    }

    // Get the underlying raw Vulkan render pass
    pub fn renderpass(&self) -> vk::RenderPass {
        self.renderpass
    }
    
    // Get the underlying raw Vulkan framebuffer
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
    } 
}