use assets::Asset;
use getset::{Getters, CopyGetters};
use gl::types::{GLuint, GLint};
use image::GenericImageView;

use crate::{basics::texture::{TextureLayout, TextureFilter, TextureWrapMode, TextureFlags, get_ifd, TextureParams, Texture, get_texel_byte_size, TextureBytes, ResizableTexture, TextureStorage}, object::PipelineElement};

// A simple two dimensional OpenGL texture
#[derive(Default, Getters, CopyGetters)]
pub struct Texture2D {
    // Storage
    storage: Option<TextureStorage>,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Vec2<u16>,
}

impl Texture for Texture2D {
    type Dimensions = vek::Vec2<u16>;

    fn target(&self) -> GLuint {
        self.storage.as_ref().expect("OpenGL Texture2D is invalid!").target()
    }
    fn texture(&self) -> GLuint {
        self.storage.as_ref().expect("OpenGL Texture2D is invalid!").name()
    }    
    fn params(&self) -> &TextureParams {
        &self.params
    }
    fn count_texels(&self) -> usize {
        self.dimensions().as_::<usize>().product()
    }
    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions   
    }
}

impl ResizableTexture for Texture2D {
    fn resize(&mut self, dimensions: Self::Dimensions) {
        todo!()
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
    pub fn dimensions(mut self, dimensions: vek::Vec2<u16>) -> Self {
        self.inner.dimensions = dimensions;
        self
    }
    // Build
    pub fn build(self) -> Texture2D {
        self.inner
    }
}

impl PipelineElement for Texture2D {
    fn add(self, pipeline: &mut crate::pipeline::Pipeline) -> crate::pipeline::Handle<Self> {
        pipeline.textures.insert(self)
    }

    fn find<'a>(pipeline: &'a crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a Self> {
        pipeline.textures.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a mut Self> {
        pipeline.textures.get_mut(handle)
    }

    fn disposed(self) {
        unsafe {
            
        }
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
                .dimensions(vek::Vec2::new(width, height))
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