use crate::{
    context::Context,
    object::{ToGlName, ToGlType, Name},
};
use ahash::AHashMap;
use std::{cell::Cell, marker::PhantomData, num::NonZeroU32};

use super::{Introspection, Uniforms};

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    pub(super) program: Name,

    // Cached texture units
    pub(super) texture_units: AHashMap<&'static str, u32>,

    // Cached binding points
    pub(super) binding_points: AHashMap<&'static str, u32>,

    // Cached pre-fetched uniform locations
    pub(super) uniform_locations: AHashMap<String, u32>,

    // Unsend and unsync lul
    pub(super) _phantom: PhantomData<*const ()>,
}

impl Program {
    // Get the uniforms of the currently bound program so we can modify them
    pub fn uniforms(&mut self, ctx: &mut Context) -> Uniforms {
        Uniforms(self)
    }
}

impl ToGlName for Program {
    fn name(&self) -> u32 {
        self.program.id()
    }
}

impl ToGlType for Program {
    fn target(&self) -> u32 {
        // This is technically not right, but it is rather a hack to keep the fexibility of the state tracker
        gl::PROGRAM
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program.id()) }
    }
}
