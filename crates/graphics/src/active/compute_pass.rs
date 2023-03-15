use wgpu::CommandEncoder;

use crate::{
    ActiveGraphicsPipeline, BufferInfo, ColorLayout,
    DepthStencilLayout, Graphics, GraphicsPipeline, RenderCommand,
    TriangleBuffer, Vertex, VertexBuffer, ComputeCommand, ComputePipeline, ActiveComputePipeline,
};
use std::{marker::PhantomData, ops::Range, sync::Arc};

// An active compute pass is basically just a wrapper that we will use to bind compute pipelines to
pub struct ActiveComputePass<'r> {
    pub(crate) commands: Vec<ComputeCommand<'r>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) push_constants: Vec<u8>,
}

impl<'r> ActiveComputePass<'r> {
    // Bind a compute pipeline, which takes mutable access of the compute pass temporarily
    // Returns an active compute pipeline that we can dispatch
    pub fn bind_pipeline<'a>(
        &'a mut self,
        pipeline: &'r ComputePipeline,
    ) -> ActiveComputePipeline<'a, 'r> {
        self.commands.push(ComputeCommand::BindPipeline(&pipeline));
        let cache = &self.graphics.0.cached;

        // Get the empty placeholder bind group
        let empty_bind_group =
            cache.bind_groups.get(&Vec::new()).unwrap();

        // Get the bind group layouts from the reflected shader
        let reflected = &pipeline.shader().reflected;
        let iter = reflected
            .bind_group_layouts
            .iter()
            .enumerate()
            .take(reflected.last_valid_bind_group_layout);

        // Set the empty bind groups for bind group layouts
        // that have been hopped over during reflection
        for (index, bind_group_layout) in iter {
            if bind_group_layout.is_none() {
                self.commands.push(ComputeCommand::SetBindGroup(
                    index as u32,
                    empty_bind_group.clone(),
                ))
            }
        }

        ActiveComputePipeline {
            pipeline: &pipeline,
            graphics: self.graphics,
            commands: &mut self.commands,
            push_constant_global_offset: self.push_constants.len(),
            push_constant: &mut self.push_constants,
        }
    }
}

impl<'r> Drop for ActiveComputePass<'r> {
    fn drop(&mut self) {
        // Create a new command encoder for this pass
        let mut encoder = self.graphics.acquire();

        // We actually record the compute pass at the very end of this wrapper
        let pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

        // Put the recorded compute pass commands in the actual compute pass
        let push_constants = std::mem::take(&mut self.push_constants);
        super::record_compute_commands(pass, push_constants, &self.commands);

        // Submit (reuse) the given encoder
        self.graphics.reuse([encoder]);
    }
}
