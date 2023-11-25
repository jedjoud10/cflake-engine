use wgpu::CommandEncoder;
use std::{marker::PhantomData, ops::Range, sync::Arc};
use crate::{pass::{ColorLayout, DepthStencilLayout}, pipeline::RenderPipeline};
use super::ActiveRenderPipeline;

// An active render pass is basically just a rasterizer that is used to bind
// multiple render pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<'r, 't, C: ColorLayout, DS: DepthStencilLayout> {
    pub(crate) inner: wgpu::RenderPass<'r>,
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
        // TODO: Check if wgpu does this internally anyways
        if Some(pipeline.pipeline().global_id()) != self.last_set_pipeline_id {
            self.inner.set_pipeline(pipeline.pipeline());
            self.last_set_pipeline_id = Some(pipeline.pipeline().global_id());
        }

        /*
        
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
        
         */

        ActiveRenderPipeline {
            inner: &mut self.inner,
            pipeline,            
            set_groups_bitflags: 0,
            set_vertex_buffer_slots: 0,
            set_index_buffer: false,
            reflected_groups_bitflags: 0,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }
}