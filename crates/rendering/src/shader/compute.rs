

use crate::context::Context;

use super::{Program, Uniforms};

// A compute shader that has a specific set of "inputs" and "outputs"
// This shall execute code on the GPU efficiently, in parallel
pub struct ComputeShader(pub(super) Program);

impl ComputeShader {
    // Create a new scheduler for this compute shader and it's corresponding uniform values
    pub fn scheduler<'s, 'c>(
        &'s mut self,
        ctx: &'c mut Context,
    ) -> (ComputeScheduler<'c>, Uniforms<'s>) {
        let uniforms = Uniforms::new(&mut self.0, ctx);
        let scheduler = ComputeScheduler { ctx };
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
pub struct ComputeScheduler<'c> {
    ctx: &'c mut Context,
}

impl<'c> ComputeScheduler<'c> {
    /*
    // Execute the compute scheduler without verifying the state of the uniforms
    pub unsafe fn run_unchecked(&mut self, axii: vek::Vec3<u32>, uniforms: &mut Uniforms) {

    }
    
    // Execute the compute shceduler by verifying the state of the uniforms
    pub fn run(&mut self, axii: vek::Vec3<u32>, uniforms: &mut Uniforms) {

    }

    // Execute the compute scheduler with the compute shader axii parameter and the valid uniforms
    pub fn run(&mut self, axii: vek::Vec3<u32>, uniforms: &mut Uniforms) -> Result<(), ()> {
        // Return an error if any of the axii is 0
        if axii.reduce_min() == 0 {
            return Err(());
        }

        // Validate the uniforms
        //uniforms.execute().unwrap();

        // Execute le compute
        Ok(())
    }
    */
}
