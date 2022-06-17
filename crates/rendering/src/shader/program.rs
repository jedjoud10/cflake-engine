use crate::{
    object::{ToGlName, ToGlTarget},
    prelude::Sampler,
};
use ahash::AHashMap;
use std::marker::PhantomData;
use world::resources::Storage;

use super::{Uniforms, Introspection};

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The OpenGL name of the program
    pub(super) name: u32,

    // Complete shader introspection (even though the values are stored directly in the following fields)
    pub(super) introspection: Introspection,

    // The texture units, alongside the name of the texture uniform that they are bound to
    pub(super) texture_units: AHashMap<&'static str, u32>,

    // A list of binding points that are created during shader compilation time
    pub(super) binding_points: AHashMap<String, u32>,

    // A list of uniform location that are created during shader compilation time
    pub(super) uniform_locations: AHashMap<String, u32>,

    // This keeps track of the total number of user defined inputs that we must set through the Uniforms struct
    // If we set the uniforms and forget some uniforms, we will crash the program. Lul
    pub(super) inputs: u32,

    // Unsync + unsend
    pub(super) _phantom: PhantomData<*const ()>,
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
