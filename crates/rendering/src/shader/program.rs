use std::{num::NonZeroU32, marker::PhantomData};
use ahash::AHashMap;
use crate::context::{ToGlName, Context, Bind, Active};

// Cached program mappings
pub(super) struct Mappings {
    // Cached uniform locations for performance
    pub(super) locations: AHashMap<String, u32>,

    // Cached uniform binding points
    pub(super) bindings: AHashMap<String, u32>,
}

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    program: NonZeroU32,

    // Le cached
    pub(super) mappings: Mappings,

    // Unsend and unsync lul
    _phantom: PhantomData<*const ()>,
}

impl Bind for Program {
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        unsafe {
            gl::UseProgram(self.program.get());
            function(Active {
                inner: self,
                context: _ctx,
            });
        }
    }

}

impl<'borrow, 'bound: 'borrow> Active<'bound, Program> {
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
