use crate::{
    context::Context,
    object::{Active, Bind, ToGlName, ToGlType},
};
use ahash::AHashMap;
use std::{cell::Cell, marker::PhantomData, num::NonZeroU32};

use super::{Introspection, Uniforms};

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    pub(super) program: NonZeroU32,

    // Cached texture units
    pub(super) texture_units: AHashMap<&'static str, u32>,

    // Cached binding points
    pub(super) binding_points: AHashMap<&'static str, u32>,

    // Cached pre-fetched uniform locations
    pub(super) uniform_locations: AHashMap<String, u32>,

    // Unsend and unsync lul
    pub(super) _phantom: PhantomData<*const ()>,
}

impl Bind for Program {
    unsafe fn bind_raw_unchecked(&mut self, ctx: &mut Context) {
        gl::UseProgram(self.target())
    }
}

impl<'bound> Active<'bound, Program> {
    // Get the uniforms of the currently bound program so we can modify them
    pub fn uniforms<'uniforms>(&'uniforms mut self) -> Uniforms<'uniforms, 'bound> {
        Uniforms(self)
    }

    // Fetch the location of a single uniform using it's name
    pub fn uniform_location(&mut self, name: &'static str) -> Option<u32> {
        todo!()
        /*
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
        */
    }
}

impl ToGlName for Program {
    fn name(&self) -> NonZeroU32 {
        self.program
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
        unsafe { gl::DeleteProgram(self.program.get()) }
    }
}
