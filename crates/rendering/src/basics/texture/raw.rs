use super::{get_ifd, TextureFlags, TextureFormat, TextureParams};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;

// Underlying texture storage
#[derive(Getters, CopyGetters)]
pub struct RawTexture {
    // The OpenGL ID for this texture
    pub name: GLuint,
    pub target: GLuint,
    // The Internal Format, Format, Data Type
    pub ifd: (GLuint, GLuint, GLuint),
}

impl RawTexture {
    // Generate a new texture, but don't put anything in it yet
    pub(crate) unsafe fn new(target: GLuint, params: &TextureParams) -> Self {
        // Create and bind
        let mut name = 0;
        gl::GenTextures(1, &mut name);
        gl::BindTexture(target, name);

        // Convert the textures to SRGBA textures if needed
        let ifd = if params.flags.contains(TextureFlags::SRGB) && params.layout.internal_format == TextureFormat::RGBA8R {
            // Get the IFD using a SRGBA format
            let mut ifd = get_ifd(params.layout);
            ifd.0 = gl::SRGB8_ALPHA8;
            ifd
        } else {
            // Get the IFD normally
            get_ifd(params.layout)
        };
        Self { name, target, ifd }
    }
}

impl Drop for RawTexture {
    fn drop(&mut self) {
        // Dispose of the texture
        unsafe { gl::DeleteTextures(1, &self.name) }
    }
}
