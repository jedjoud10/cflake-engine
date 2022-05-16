use std::{num::NonZeroU32, marker::PhantomData};

use ahash::AHashMap;

use crate::context::{ToGlName, Context, Bind, Active};

use super::Uniforms;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    program: NonZeroU32,

    // Cached uniform locations for performance
    uniforms: AHashMap<String, u32>,

    // Unsend and unsync lul
    _phantom: PhantomData<*const ()>,
}

impl Program {
    // Get the cached uniform mappings immutably
    pub fn cached_mappings(&self) -> &AHashMap<String, u32> {
        &self.uniforms
    }

    // Get the cached uniform mappings mutably
    pub fn cached_mappings_mut(&mut self) -> &mut AHashMap<String, u32> {
        &mut self.uniforms
    }
}

impl<'a> Bind<'a> for Program {
    type Bound = ;
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        unsafe {
            gl::UseProgram(self.program.get());
            function(Active(self));
        }
    }

}

impl<'a> Active<'a, Program> {
    // Create a uniforms setter using a bound program
    pub fn uniforms(&'a mut self) -> Uniforms<'a> {
        Uniforms(self)
    } 
}

impl ToGlName for Program {
    fn name(&self) -> NonZeroU32 {
        self.program
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program.get())
        }
    }
}
