use crate::{context::Context, object::Active};

use super::{ComputeStage, Program};

// A compute shader that has a specific set of "inputs" and "outputs"
// This shall execute code on the GPU efficiently, in parallel
pub struct ComputeShader(pub(super) Program);

impl AsRef<Program> for ComputeShader {
    fn as_ref(&self) -> &Program {
        &self.0
    }
}

impl AsMut<Program> for ComputeShader {
    fn as_mut(&mut self) -> &mut Program {
        &mut self.0
    }
}

impl<'a> Active<'a, ComputeShader> {    
}