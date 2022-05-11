use std::num::NonZeroU32;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    program: NonZeroU32,
}