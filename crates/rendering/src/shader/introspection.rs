use crate::context::Context;

use super::Program;

// A single block, can represent a uniform block or an SSBO block
pub struct Block {
    // Binding point of this block
    binding: u32,

    // Byte size of this block
    size: Option<usize>,

    // Full name of this block
    name: String,
}

// Shader introspection is how we can fetch the shader block binding points and such
pub struct Introspection {
    // Normal uniform blocks
    uniform_blocks: Vec<Block>,

    // Shader storage blocks
    ssbo_blocks: Vec<Block>,
}

// Introspect a shader, and construct an Introspection struct
pub(super) fn introspect(ctx: &mut Context, shader: impl AsRef<Program>) -> Introspection {

}