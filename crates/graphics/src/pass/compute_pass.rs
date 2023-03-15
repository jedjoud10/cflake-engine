use wgpu::CommandEncoder;

use crate::{
    ActiveRenderPass, ColorAttachments, ColorLayout, ColorOperations,
    DepthStencilAttachment, DepthStencilLayout,
    DepthStencilOperations, Graphics, GraphicsPipeline, Vertex,
    VertexBuffer, ActiveComputePass,
};
use std::marker::PhantomData;

// Wrapper around a WGPU compute pass
// You don't write to anything to begin with so it's not even that complex
pub struct ComputePass {
    graphics: Graphics,
}

impl ComputePass {
    // Create a new compute pass to be used later on (not really)
    pub fn new(
        graphics: &Graphics,
    ) -> Self {
        Self {
            graphics: graphics.clone(),
        }
    }

    // Begin the compute pass and return an active compute pass that we can use to bind multiple
    // compute pipelines to so we can compute some stuff on the GPU
    pub fn begin<'r>(
        &'r self,
    ) -> ActiveComputePass {
        ActiveComputePass {
            commands: Vec::new(),
            graphics: &self.graphics,
            push_constants: Vec::new()
        }
    }
}
