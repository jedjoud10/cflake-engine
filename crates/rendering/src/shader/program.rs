use crate::{
    context::Context,
    object::{Active, Bind, ToGlName},
};
use ahash::AHashMap;
use std::{marker::PhantomData, num::NonZeroU32};

use super::Introspection;

// Cached program mappings
#[derive(Default)]
pub(super) struct Mappings {
    // Cached uniform locations for performance
    pub(super) locations: AHashMap<String, u32>,

    // Cached block binding points
    pub(super) bindings: AHashMap<String, u32>,
}

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    pub(super) program: NonZeroU32,

    // Le cached
    pub(super) mappings: Mappings,
    pub(super) introspection: Introspection,

    // Unsend and unsync lul
    pub(super) _phantom: PhantomData<*const ()>,
}

impl Bind for Program {
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        unsafe {
            gl::UseProgram(self.program.get());
            function(Active::new(self, _ctx));
        }
    }
}

impl<'borrow, 'bound: 'borrow> Active<'bound, Program> {
    // Fetch the location of a single uniform using it's name
    pub fn fetch_uniform_location(&mut self, name: &'static str) -> Option<u32> {
        // Check if the uniform was already stored within the program
        let program = self.as_mut();

        // Either insert or fetch from OpenGL
        program.mappings.locations.get(name).cloned().or_else(|| {
            // Fetch the location from OpenGL, and insert it
            let location = unsafe {
                let name = name as *const str as *const i8;
                gl::GetUniformLocation(program.program.get(), name)
            };

            // Validate location
            (location != -1).then(|| location as u32)
        })
    }
}

impl ToGlName for Program {
    fn name(&self) -> NonZeroU32 {
        self.program
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program.get()) }
    }
}
