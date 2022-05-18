use crate::{context::Context, object::Active};
use rendering_derive::Uniform;
use super::Program;

// A uniform value that can be stored within some uniforms
pub trait UniformValue {
    // Update the uniform within the currentlty bound program
    unsafe fn set_raw_uniform_value(&self, ctx: &mut Context, name: &'static str, bound: Active<Program>);
}

impl UniformValue for u32 {
    unsafe fn set_raw_uniform_value(&self, ctx: &mut Context, name: &'static str, bound: Active<Program>) {
        todo!()
    }
}

// A uniform struct will set multiple uniform values at once
pub unsafe trait UniformStruct {
    // Set multiple uniform values at once
    unsafe fn set_uniform_values(&self, ctx: &mut Context, bound: Active<Program>);
}