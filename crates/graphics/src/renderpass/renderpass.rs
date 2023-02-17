use wgpu::CommandEncoder;

use crate::{
    ColorAttachments, ColorLayout,
    DepthStencilAttachment, DepthStencilLayout, Graphics,
    GraphicsPipeline, RenderPassBeginError,
    RenderPassInitializationError, Vertex, VertexBuffer, ColorOperations, DepthStencilOperations, ActiveRenderPass,
};
use std::marker::PhantomData;

// Wrapper around a WGPU render pass
// This render pass must be instantiated with the specific attacment layouts before we actually use it
// It's a bit more restrictive than what WGPU allows us to do, but I like it more this way since you can't
// accidentally write to a texture you weren't meant to write to
pub struct RenderPass<C: ColorLayout, DS: DepthStencilLayout> {
    color_layout_operations: Vec<wgpu::Operations<wgpu::Color>>,
    depth_operations: Option<wgpu::Operations<f32>>,
    stencil_operations: Option<wgpu::Operations<u32>>,
    _phantom_color: PhantomData<C>,
    _phantom_depth_stencil: PhantomData<DS>,
    graphics: Graphics,
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Create a new render pass for use with a specific color layout and depth stencil layout
    pub fn new(
        graphics: &Graphics,
        color_operations: impl ColorOperations<C>,
        depth_stencil_operations: impl DepthStencilOperations<DS>,
    ) -> Result<Self, RenderPassInitializationError> {
        Ok(Self {
            color_layout_operations: color_operations.operations(),
            depth_operations: depth_stencil_operations.depth_operations(),
            stencil_operations: depth_stencil_operations.stencil_operations(),
            _phantom_color: PhantomData,
            _phantom_depth_stencil: PhantomData,
            graphics: graphics.clone(),
        })
    }

    // Begin the render pass and return an active render pass that we can use to bind multiple
    // graphical pipelines to so we can render specific types of objects
    pub fn begin<'r>(
        &'r mut self,
        encoder: &'r mut CommandEncoder,
        color_attachments: impl ColorAttachments<'r, C>,
        depth_stencil_attachment: impl DepthStencilAttachment<'r, DS>,
    ) -> Result<ActiveRenderPass<C, DS>, RenderPassBeginError>{
        // Fetch the appropriate texture views to use
        let color_views = color_attachments.views();
        let depth_stencil_view = depth_stencil_attachment.view();
        
        // Extract operations that we used in the RenderPass' setup
        let color_ops = &self.color_layout_operations;
        let depth_ops = self.depth_operations;
        let stencil_ops = self.stencil_operations;

        // Get a vector that contains all RenderPassColorAttachments
        let color_attachments = color_views.iter().zip(color_ops.iter()).map(|(view, ops)| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: *ops,
            })
        }).collect::<Vec<_>>();

        // Get the Option that contains the RenderPassDepthStencilAttachment
        let depth_stencil_attachment = depth_stencil_view.map(|view| {
            wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops,
                stencil_ops,
            }
        });

        // Being the Wgpu render pass
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor::<'r, '_> {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        });

        Ok(ActiveRenderPass { 
            render_pass: pass,
            _phantom: PhantomData,
            _phantom2: PhantomData
        })
    }
}
