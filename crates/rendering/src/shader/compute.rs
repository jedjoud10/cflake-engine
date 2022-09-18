use std::marker::PhantomData;

use crate::{
    buffer::DispatchComputerIndirectBuffer,
    context::{ToGlName, ToGlTarget},
};

use super::{Program, Uniforms, ValidUniforms};

// A compute shader that has a specific set of "inputs" and "outputs"
// This shall execute code on the GPU efficiently, in parallel
pub struct ComputeShader(pub(super) Program);

impl ComputeShader {
    // Create a new scheduler for this compute shader and it's corresponding uniform values
    pub fn scheduler<'s, 'c>(&'s mut self) -> (ComputeScheduler<'s>, Uniforms<'s>) {
        let uniforms = Uniforms::new(&mut self.0);
        let scheduler = ComputeScheduler {
            _phantom: PhantomData::default(),
        };
        (scheduler, uniforms)
    }
}

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

// The responsability of a compute scheduler is to set the compute shader parameters and to execute a compute shader
pub struct ComputeScheduler<'s> {
    _phantom: PhantomData<&'s mut ComputeShader>,
}

impl<'s> ComputeScheduler<'s> {
    // Execute the compute scheduler with the compute shader axii parameter and the valid uniforms
    pub fn run(&mut self, axii: vek::Vec3<u32>, uniforms: ValidUniforms) {
        if axii.reduce_min() == 0 {
            return;
        }

        unsafe {
            gl::UseProgram(uniforms.0.name);
            gl::DispatchCompute(axii.x, axii.y, axii.z)
        }
    }

    // Execute the compute scheduler using a specific dispatch indirect buffer
    pub fn run_indirect(
        &mut self,
        buffer: &DispatchComputerIndirectBuffer,
        uniforms: ValidUniforms,
    ) {
        unsafe {
            gl::UseProgram(uniforms.0.name);
            gl::BindBuffer(DispatchComputerIndirectBuffer::target(), buffer.name());
            gl::DispatchComputeIndirect(0);
        }
    }
}
