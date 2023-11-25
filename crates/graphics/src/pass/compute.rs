use wgpu::CommandEncoder;
use std::marker::PhantomData;

use crate::{context::Graphics, active::ActiveComputePass};

// Compute pass basically just doesn't exist
pub struct ComputePass;

impl ComputePass {
    // Begin the compute pass and return an active compute pass that we can use to bind multiple
    // compute pipelines to so we can compute some stuff on the GPU
    pub fn begin(graphics: &Graphics) -> ActiveComputePass {
        todo!()
    }
}
