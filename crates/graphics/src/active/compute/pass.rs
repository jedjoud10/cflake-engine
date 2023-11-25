use std::{marker::PhantomData, ops::Range, sync::Arc};
use crate::shader::ComputeShader;
use super::ActiveComputeDispatcher;

// An active compute pass is basically just a wrapper that we will use to bind compute pipelines to
pub struct ActiveComputePass<'r> {
    pub(crate) inner: wgpu::ComputePass<'r>,
    pub(crate) last_set_pipeline_id: Option<wgpu::Id<wgpu::ComputePipeline>>,
}

impl<'r> ActiveComputePass<'r> {
    // Bind a compute shader, which takes mutable access of the compute pass temporarily
    // Returns an active compute dispatcher that we can dispatch
    pub fn bind_shader<'a>(
        &'a mut self,
        shader: &'r ComputeShader,
    ) -> ActiveComputeDispatcher<'a, 'r> {
        // If this pipeline was already set before, don't bother re-setting it
        // TODO: Check if wgpu does this internally anyways
        if Some(shader.pipeline().global_id()) != self.last_set_pipeline_id {
            self.inner.set_pipeline(&shader.pipeline);
            self.last_set_pipeline_id = Some(shader.pipeline().global_id());
        }

        todo!()
    }
}