use crate::{
    ColorAttachments, ColorLayout,
    DepthStencilAttachment, DepthStencilLayout, Graphics,
    GraphicsPipeline, RenderPassBeginError,
    RenderPassInitializationError, Vertex, VertexBuffer,
};
use std::marker::PhantomData;

// Wrapper around a WGPU render pass
// This render pass must be instantiated with the specific attacment layouts before we actually use it
// It's a bit more restrictive than what WGPU allows us to do, but I like it more this way since you can't
// accidentally write to a texture you weren't meant to write to
pub struct RenderPass<C: ColorLayout, DS: DepthStencilLayout> {
    // The render pass doesn't store anything, we just use it for strictness
    _phantom_color: PhantomData<C>,
    _phantom_depth_stencil: PhantomData<DS>,
    graphics: Graphics,
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Create a new render pass for use with a specific color layout and depth stencil layout
    pub fn new(
        graphics: &Graphics,
    ) -> Result<Self, RenderPassInitializationError> {
        todo!()
    }

    // Begin the render pass and return an active render pass that we can use to bind multiple
    // graphical pipelines to so we can render specific types of objects
    pub fn begin<'r, 'c, 'ds>(
        &'r mut self,
        color_attachments: impl ColorAttachments<'c, C>,
        depth_stencil_attachment: impl DepthStencilAttachment<'ds, DS>,
    ) {}
}
