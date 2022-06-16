use crate::{object::{ToGlName, ToGlTarget}, prelude::Sampler};
use ahash::AHashMap;
use world::resources::Storage;
use std::marker::PhantomData;

use super::Uniforms;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    pub(super) name: u32,

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
    pub fn uniforms(&mut self) -> Uniforms {
        Uniforms(self)
    }
}

impl ToGlName for Program {
    fn name(&self) -> u32 {
        self.name
    }
}

impl ToGlTarget for Program {
    fn target() -> u32 {
        gl::PROGRAM
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.name) }
    }
}
