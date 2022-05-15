use crate::{context::Context, task::Fence};

use super::{ComputeStage, Program};

// A compute shader that has a specific set of "inputs" and "outputs"
// This shall execute code on the GPU efficiently, in parallel
pub struct ComputeShader(Program);

impl AsRef<Program> for ComputeShader {
    fn as_ref(&self) -> &Program {
        &self.0
    }
}

impl ComputeShader {
    // Execute a compute shader, and return an async handle (basically just a GL fence)
    pub fn execute(&mut self, ctx: &mut Context, x: u32, y: u32, z: u32) -> Fence {
        todo!()
    }
}
