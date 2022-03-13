use assets::Asset;
use getset::{Getters, CopyGetters};
use gl::types::{GLuint, GLint};
use image::GenericImageView;

use crate::{basics::texture::{TextureLayout, TextureFilter, TextureWrapMode, TextureFlags, get_ifd, TextureParams, Texture, get_texel_byte_size, TextureBytes}, object::PipelineElement};

// A simple two dimensional OpenGL texture
#[derive(Default, Getters, CopyGetters)]
pub struct Texture2D {
    // The OpenGL id for this texture
    #[getset(get_copy = "pub(crate)")]
    buffer: GLuint,

    // Params
    params: TextureParams,

    // Texture dimensions
    #[getset(get_copy = "pub")]
    width: u16,
    #[getset(get_copy = "pub")]
    height: u16,
}

impl Texture for Texture2D {
    fn target(&self) -> GLuint {
        gl::TEXTURE_2D
    }
    fn texture(&self) -> GLuint {
        self.buffer
    }    
    fn params(&self) -> &TextureParams {
        &self.params
    }
    fn count_texels(&self) -> usize {
        self.width as usize * self.height as usize
    }
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

impl PipelineElement for Texture2D {
    fn add(self, pipeline: &mut crate::pipeline::Pipeline) -> crate::pipeline::Handle<Self> {
        todo!()
    }

    fn find<'a>(pipeline: &'a crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a Self> {
        todo!()
    }

    fn find_mut<'a>(pipeline: &'a mut crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a mut Self> {
        todo!()
    }

    fn disposed(self) {
        todo!()
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
                .dimensions(width, height)
                .params(TextureParams {
                    bytes: TextureBytes::Loaded(bytes),
                    layout: TextureLayout::default(),
                    filter: TextureFilter::Linear,
                    wrap: TextureWrapMode::ClampToEdge(None),
                    flags: TextureFlags::MIPMAPS | TextureFlags::SRGB,
                })
                .build(),
        )
    }
}