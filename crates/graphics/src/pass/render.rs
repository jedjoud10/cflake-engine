use wgpu::CommandEncoder;
use std::{marker::PhantomData, f32::consts::E};

use crate::active::ActiveRenderPass;

use super::{ColorLayout, DepthStencilLayout, ColorOperations, DepthStencilOperations, ColorAttachments, DepthStencilAttachment};

// Wrapper around a WGPU render pass
// This render pass must be instantiated with the specific attachment layouts before we actually use it
// It's a bit more restrictive than what WGPU allows us to do, but I like it more this way since you can't
// accidentally write to a texture you weren't meant to write to
// Also, it resembles more how Vulkan render passes work, and I like how they force you to write the operations before hand anyways
pub struct RenderPass<C: ColorLayout, DS: DepthStencilLayout> {
    color_layout_operations: Vec<wgpu::Operations<wgpu::Color>>,
    depth_operations: Option<wgpu::Operations<f32>>,
    stencil_operations: Option<wgpu::Operations<u32>>,
    _phantom_color: PhantomData<C>,
    _phantom_depth_stencil: PhantomData<DS>,
}

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPass<C, DS> {
    // Create a new render pass for use with a specific color layout and depth stencil layout
    pub fn new(
        color_operations: impl ColorOperations<C>,
        depth_stencil_operations: impl DepthStencilOperations<DS>,
    ) -> Self {
        Self {
            color_layout_operations: color_operations.operations(),
            depth_operations: depth_stencil_operations.depth_operations(),
            stencil_operations: depth_stencil_operations.stencil_operations(),
            _phantom_color: PhantomData,
            _phantom_depth_stencil: PhantomData,
        }
    }

    // Begin the render pass and return an active render pass that we can use to bind multiple
    // graphical pipelines to so we can render specific types of objects
    pub fn begin<'a: 'r, 'r>(
        &'r mut self,
        encoder: &'a mut wgpu::CommandEncoder,
        color_attachments: impl ColorAttachments<'r, C>,
        depth_stencil_attachment: impl DepthStencilAttachment<'r, DS>,
    ) -> ActiveRenderPass<C, DS> {
        // Fetch the appropriate texture views to use
        let color_views = color_attachments.views();
        let depth_stencil_view = depth_stencil_attachment.view();

        // Extract operations that we used in the RenderPass' setup
        let color_ops = &self.color_layout_operations;
        let depth_ops = self.depth_operations;
        let stencil_ops = self.stencil_operations;

        // Get a vector that contains all RenderPassColorAttachments
        let color_attachments = color_views
            .iter()
            .zip(color_ops.iter())
            .map(|(view, ops)| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: *ops,
                })
            })
            .collect::<Vec<_>>();

        // Get the Option that contains the RenderPassDepthStencilAttachment
        let depth_stencil_attachment =
            depth_stencil_view.map(|view| wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops,
                stencil_ops,
            });

        let inner = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        ActiveRenderPass {
            _phantom: PhantomData,
            _phantom2: PhantomData,
            last_set_pipeline_id: None,
            inner,
        }
    }
}
