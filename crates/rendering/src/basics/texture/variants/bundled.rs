use getset::{Getters, CopyGetters};
use gl::types::{GLuint, GLint};
use crate::basics::texture::{TextureLayout, TextureWrapMode, TextureBits, TextureFilter, TextureParams, Texture, get_texel_byte_size};
use super::{Texture2D, TextureBuilder};

// Bundled bytes
pub type BundledBytes = Option<Vec<u8>>;

// A combination of multiple texture2D's
// This represents an OpenGL Array Texture
#[derive(Getters, CopyGetters)]
pub struct BundledTexture2D {
    // The OpenGL id for this texture
    buffer: GLuint,
    // The bytes stored in this texture
    #[getset(get = "pub")]
    bytes: BundledBytes,

    // Params
    #[getset(get = "pub")]
    params: TextureParams,

    // Texture dimensions
    #[getset(get_copy = "pub")]
    width: u16, 
    #[getset(get_copy = "pub")]
    height: u16,
    #[getset(get_copy = "pub")]
    layers: u16
}

impl Texture for BundledTexture2D {
    fn texture(&self) -> GLuint {
        self.buffer
    }
    fn count_texels(&self) -> usize {
        self.width * self.height * self.layers
    }
    fn count_bytes(&self) -> usize {
        self.count_texels() * get_texel_byte_size(self.params.layout.internal_format)
    }

    // Initialize the bundled texture
    fn init(&mut self) {
        
    }
}


// A texture bundler that creates a 2D texture array from a set of textures
#[derive(Default)]
pub struct BundledTextureBuilder {
    textures: Vec<Texture2D>
}
impl BundledTextureBuilder {
    // Add a texture to the bundler
    pub fn push(mut self, texture: Texture2D) -> Self {
        self.textures.push(texture);
        self
    }
    // Build the bundled texture
    pub fn build(mut self) -> Option<BundledTexture2D> {
        // We get the main dimensions from the first texture
        let first = self.textures.get(0)?;
        let (width, height) = (first.width(), first.height());

        // Load the bytes
        let mut bytes: Vec<u8> = Vec::with_capacity(self.textures[0].count_bytes());
        for texture in self.textures {
            // Check if we have the same settings
            if texture.width() != width || texture.height() != height {
                return None;
            }
            bytes.extend(texture.bytes().iter());
        }
        Some(BundledTexture2D {
            buffer: 0,
            bytes,
            params: first.params(),
            width,
            height,
            layers: self.textures.len(),
        })
    }
}
