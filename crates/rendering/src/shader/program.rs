use std::num::NonZeroU32;

use crate::context::ToGlName;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    program: NonZeroU32,
}

impl ToGlName for Program {
    fn name(&self) -> NonZeroU32 {
        self.program
    }
}
