use super::SourceStage;
use std::num::NonZeroU32;

// A program is the underlying compiled shader that we will store inside the shader wrappers
pub struct Program {
    // The program OpenGL name
    program: NonZeroU32,

    // How many sources were linked to this program, and each of their types
    sources: Vec<SourceStage>,
}
