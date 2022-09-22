use super::{Introspection, BlockIndex};
use crate::context::{ToGlName, ToGlTarget};
use ahash::AHashMap;
use std::marker::PhantomData;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The OpenGL name of the program
    pub(super) name: u32,

    // The user friendly name of the program
    pub(super) username: String,

    // Complete shader introspection (even though the values are stored directly in the following fields)
    pub(super) introspection: Introspection,
    pub(super) buffer_block_locations: AHashMap<String, BlockIndex>,
    pub(super) uniform_locations: AHashMap<String, u32>,

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
