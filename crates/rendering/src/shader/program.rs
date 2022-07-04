use super::Introspection;
use crate::object::{ToGlName, ToGlTarget};
use ahash::{AHashMap, AHashSet};
use std::marker::PhantomData;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The OpenGL name of the program
    pub(super) name: u32,

    // The user friendly name of the program
    pub(super) username: String,

    // Complete shader introspection (even though the values are stored directly in the following fields)
    pub(super) introspection: Introspection,

    // The texture units, alongside the name of the texture uniform that they are bound to
    pub(super) texture_units: AHashMap<&'static str, u32>,

    // A list of binding points (for buffers / ssbos) that are created during shader compilation time
    // The boolean tells us if the user set the binding point or not
    pub(super) binding_points: AHashMap<String, (u32, bool)>,

    // A list of uniform location that are created during shader compilation time
    // The boolean tells us if the user set the uniform or not
    pub(super) uniform_locations: AHashMap<String, (u32, bool)>,

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
