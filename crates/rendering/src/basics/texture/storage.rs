use getset::{Getters, CopyGetters};
use gl::types::{GLint, GLuint};
use super::{TextureLayout, TextureFlags, TextureParams};


// Underlying texture storage
#[derive(Getters, CopyGetters)]
pub struct TextureStorage {
    // The OpenGL ID for this texture
    #[getset(get_copy = "pub")]
    name: GLuint,
    #[getset(get_copy = "pub")]
    target: GLuint,
    // The Internal Format, Format, Data Type
    ifd: (GLint, GLuint, GLuint)
}

impl TextureStorage {
    // Initialize some new storage
    pub fn new(target: GLuint, params: &TextureParams) -> Self {
        todo!()
    }
}