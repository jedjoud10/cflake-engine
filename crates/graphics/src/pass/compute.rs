use wgpu::CommandEncoder;

use crate::{
    ActiveComputePass, ActiveRenderPass, ColorAttachments, ColorLayout, ColorOperations,
    DepthStencilAttachment, DepthStencilLayout, DepthStencilOperations, Graphics, RenderPipeline,
    Vertex, VertexBuffer,
};
use std::marker::PhantomData;

// Compute pass basically just doesn't exist
pub struct ComputePass;

impl ComputePass {
    // Begin the compute pass and return an active compute pass that we can use to bind multiple
    // compute pipelines to so we can compute some stuff on the GPU
    pub fn begin(graphics: &Graphics) -> ActiveComputePass {
        ActiveComputePass {
            commands: Vec::new(),
            graphics: graphics,
            push_constants: Vec::new(),
            last_set_pipeline_id: None,
        }
    }
}
