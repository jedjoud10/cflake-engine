use assets::Asset;
use getset::{Getters, CopyGetters};
use gl::types::{GLuint, GLint};

use crate::basics::texture::{TextureLayout, TextureFilter, TextureWrapMode, TextureBits, get_ifd, TextureParams};

// Loaded bytes
pub type LoadedBytes = Option<Vec<u8>>;
// A simple two dimensional OpenGL texture
#[derive(Default, Getters, CopyGetters)]
pub struct Texture2D {
    // The OpenGL id for this texture
    #[getset(get_copy = "pub(crate)")]
    buffer: GLuint,
    // The bytes stored in this texture
    #[getset(get = "pub")]
    bytes: LoadedBytes,

    // Params
    #[getset(get = "pub")]
    params: TextureParams,

    // Texture dimensions
    #[getset(get_copy = "pub")]
    width: u16,
    #[getset(get_copy = "pub")]
    height: u16,
}

// Builder
#[derive(Default)]
pub struct TextureBuilder {
    inner: Texture2D,
}

impl TextureBuilder {
    // Create a new builder from a texture
    pub fn new(texture: Texture2D) -> Self {
        Self { inner: texture }
    }
    // This burns my eyes
    pub fn bytes(mut self, bytes: Vec<u8>) -> Self {
        self.inner.bytes = bytes;
        self
    }
    pub fn params(mut self, params: TextureParams) -> Self {
        self.inner.params = params;
        self
    }
    pub fn dimensions(mut self, width: u16, height: u16) -> Self {
        self.inner.width = width;
        self.inner.height = height;
        self
    }
    // Build
    pub fn build(self) -> Texture2D {
        self.inner
    }
}

// Load a Texture2D
impl Asset for Texture2D {
    fn deserialize(self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let image = image::load_from_memory(bytes).unwrap();
        let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
        // Flip
        let image = image.flipv();
        let (bytes, width, height) = (image.to_bytes(), image.width() as u16, image.height() as u16);
        Some(
            TextureBuilder::default()
                .bytes(bytes)
                .dimensions(width, height)
                .params(TextureParams {
                    layout: TextureLayout::default(),
                    filter: TextureFilter::Linear,
                    wrap: TextureWrapMode::ClampToEdge(None),
                    bits: TextureBits::MIPMAPS | TextureBits::SRGB,
                })
                .build(),
        )
    }
}