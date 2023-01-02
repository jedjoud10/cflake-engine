use vulkan::Recorder;
use crate::{RenderPass, Graphics};

// This is an active render pass that is currently
// able to render to the framebuffer attachments
pub struct ActiveRenderPass<'a, 'r> {
    pub(super) renderpass: &'a RenderPass,
    pub(super) recorder: &'a mut Recorder<'r>,
}

impl<'a, 'r> Drop for ActiveRenderPass<'a, 'r> {
    fn drop(&mut self) {
        let graphics = Graphics::global();
    }
}