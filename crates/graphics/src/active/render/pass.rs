use wgpu::CommandEncoder;

use crate::{
    calculate_refleced_group_bitset, ActiveRenderPipeline, BufferInfo, ColorLayout,
    DepthStencilLayout, Graphics, RenderCommand, RenderPipeline, TriangleBuffer, Vertex,
    VertexBuffer,
};
use std::{marker::PhantomData, ops::Range, sync::Arc};

// An active render pass is basically just a rasterizer that is used to bind
// multiple render pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<'r, 't, C: ColorLayout, DS: DepthStencilLayout> {
    pub(crate) commands: Vec<RenderCommand<'r, C, DS>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) push_constants: Vec<u8>,
    pub(crate) color_attachments: Vec<Option<wgpu::RenderPassColorAttachment<'t>>>,
    pub(crate) depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'t>>,
    pub(crate) last_set_pipeline_id: Option<wgpu::Id<wgpu::RenderPipeline>>,
    pub(crate) _phantom: PhantomData<&'t C>,
    pub(crate) _phantom2: PhantomData<&'t DS>,
}

impl<'r, 't, C: ColorLayout, DS: DepthStencilLayout> ActiveRenderPass<'r, 't, C, DS> {
    // Bind a graphics pipeline, which takes mutable access of the render pass temporarily
    // Returns an active graphics pipeline that we can render to
    pub fn bind_pipeline<'a>(
        &'a mut self,
        pipeline: &'r RenderPipeline<C, DS>,
    ) -> ActiveRenderPipeline<'a, 'r, 't, C, DS> {
        // If this pipeline was already set before, don't bother re-setting it
        if Some(pipeline.pipeline().global_id()) != self.last_set_pipeline_id {
            self.commands.push(RenderCommand::BindPipeline(&pipeline));
            let cache = &self.graphics.0.cached;

            // Get the empty placeholder bind group
            let empty_bind_group = cache.bind_groups.get(&Vec::new()).unwrap();

            // Get the bind group layouts from the reflected shader
            let reflected = &pipeline.shader().reflected;
            let iter = reflected
                .bind_group_layouts
                .iter()
                .enumerate()
                .take(reflected.taken_bind_group_layouts);

            // Set the empty bind groups for bind group layouts
            // that have been hopped over during reflection
            for (index, bind_group_layout) in iter {
                if bind_group_layout.is_none() {
                    self.commands.push(RenderCommand::SetBindGroup(
                        index as u32,
                        empty_bind_group.clone(),
                    ))
                }
            }

            self.last_set_pipeline_id = Some(pipeline.pipeline().global_id());
        }

        ActiveRenderPipeline {
            _phantom: PhantomData,
            _phantom2: PhantomData,
            pipeline: &pipeline,
            graphics: self.graphics,
            commands: &mut self.commands,
            push_constant_global_offset: self.push_constants.len(),
            push_constant: &mut self.push_constants,
            set_groups_bitflags: 0,
            reflected_groups_bitflags: calculate_refleced_group_bitset(
                &pipeline.shader().reflected,
            ),
            set_vertex_buffer_slots: 0,
            set_index_buffer: false,
        }
    }
}

impl<'r, 't, C: ColorLayout, DS: DepthStencilLayout> Drop for ActiveRenderPass<'r, 't, C, DS> {
    fn drop(&mut self) {
        // Create a new command encoder for this pass
        let mut encoder = self.graphics.acquire();

        // We actually record the render pass at the very end of this wrapper
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &self.color_attachments,
            depth_stencil_attachment: self.depth_stencil_attachment.take(),
        });

        // Put the recorded render pass commands in the actual render pass
        let push_constants = std::mem::take(&mut self.push_constants);
        super::record_render_commands(pass, push_constants, &self.commands);

        // Submit (reuse) the given encoder
        self.graphics.reuse([encoder]);
    }
}
