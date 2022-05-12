use std::num::NonZeroU32;

use assets::Asset;

use crate::context::Context;

// A single shader source that can make up a bigger shader. A source is usually a single text file ending with .glsl
pub struct Source {
    // The filtered shader source text (without any directives)
    txt: String,

    // OpenGL type for this shader source
    gl_type: NonZeroU32,
}

impl<'a> Asset<'a> for Source {
    type OptArgs = &'a mut Context;

    fn is_extension_valid(ext: &str) -> bool {
        ext == "vert.glsl" || ext == "frag.glsl" || ext == "cmpt.glsl"
    }

}