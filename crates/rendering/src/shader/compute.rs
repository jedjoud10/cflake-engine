use crate::context::Context;

use super::ComputeStage;

// A compute shader that has a specific set of "inputs" and "outputs"
// This shall execute code on the GPU efficiently, in parallel
pub struct ComputeShader {
    // Compute shaders only have one stage
    stage: ComputeStage,
}


impl ComputeShader {
    // Execute a compute shader, and return an async handle (basically just a GL fence)
    pub fn execute(&mut self, ctx: &mut Context, x: u32, y: u32, z: u32) -> Fence {

    }
}