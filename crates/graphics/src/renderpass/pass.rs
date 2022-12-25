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
        // Create a default framebuffer
        let extent = graphics.swapchain().extent();
        let framebuffer = Self::create_framebuffer(&graphics, extent);
        let renderpass_create_info = vk::RenderPassCreateInfo::builder();

        todo!()
    }

    // Create a framebuffer for the given extent
    fn create_framebuffer(graphics: &Graphics, extent: vek::Extent2<u32>) -> vk::Framebuffer {
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

    // Recreate the render pass and modify it's framebuffer size
    pub unsafe fn resize(
        &mut self,
        dimensions: vek::Extent2<u32>,
    ) {
        self.graphics.device().wait();
        
        // Create a new framebuffer
        let framebuffer = Self::create_framebuffer(&self.graphics, dimensions);

        // Replace the framebuffer
        let old = std::mem::replace(&mut self.framebuffer, framebuffer);        

        // Destroy the old framebuffer
        self.graphics.device().destroy_framebuffer(old);
    }
}