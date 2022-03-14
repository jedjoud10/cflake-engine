use std::ptr::null;

use assets::Asset;
use getset::{CopyGetters, Getters};
use gl::types::{GLint, GLuint};
use image::GenericImageView;

use crate::{
    basics::texture::{
        generate_filters, generate_mipmaps, get_ifd, get_texel_byte_size, guess_mipmap_levels, store_bytes, verify_byte_size, RawTexture, ResizableTexture, Texture, TextureBytes,
        TextureFilter, TextureFlags, TextureLayout, TextureParams, TextureWrapMode,
    },
    object::PipelineElement,
};

// A simple two dimensional OpenGL texture
#[derive(Default, Getters, CopyGetters)]
pub struct Texture2D {
    // Storage
    raw: Option<RawTexture>,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Vec2<u16>,
}

impl Texture for Texture2D {
    type Dimensions = vek::Vec2<u16>;
    fn storage(&self) -> Option<&RawTexture> {
        self.raw.as_ref()
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
    fn write(&mut self, bytes: Vec<u8>) {
        // Write to the OpenGL texture first
        let ptr = verify_byte_size(self.count_bytes(), &bytes);

        // Write
        if let Some(raw) = self.raw.as_ref() {
            let (width, height) = (self.dimensions).as_::<i32>().into_tuple();
            let ifd = raw.ifd;
            unsafe {
                gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width, height, ifd.1, ifd.2, ptr);
            }
        }

        // Then save the bytes if possible
        store_bytes(self.params().flags, bytes, &mut self.params.bytes);
    }
}

impl ResizableTexture for Texture2D {
    fn resize_then_write(&mut self, dimensions: vek::Vec2<u16>, bytes: Vec<u8>) {
        // Check if we can even resize the texture
        assert!(self.params.flags.contains(TextureFlags::RESIZABLE), "Texture cannot be resized!");

        // Resize the texture
        self.dimensions = dimensions;
        let (width, height) = dimensions.as_::<i32>().into_tuple();
        // This will also automatically clear the image
        let raw = self.raw.as_ref();
        if let Some(raw) = raw {
            unsafe {
                let ptr = verify_byte_size(self.count_bytes(), &bytes);
                gl::TexImage2D(gl::TEXTURE_2D, 0, raw.ifd.0 as i32, width, height, 0, raw.ifd.1, raw.ifd.2, ptr);
            }
        }
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
    fn add(mut self, pipeline: &mut crate::pipeline::Pipeline) -> crate::pipeline::Handle<Self> {
        // Create the raw texture wrapper
        let (texture, ptr) = unsafe { RawTexture::new(gl::TEXTURE_2D, &self.params) };
        let ifd = texture.ifd;
        let (width, height) = self.dimensions.as_::<i32>().into_tuple();

        // Texture generation, SRGB, mipmap, filters
        unsafe {
            // Get the byte size per texel
            let tsize = get_texel_byte_size(self.params.layout.internal_format) as isize;

            // Depends if it is resizable or not
            if self.params.flags.contains(TextureFlags::RESIZABLE) {
                // Dynamic
                gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0 as i32, width, height, 0, ifd.1, ifd.2, ptr);
            } else {
                // Static
                gl::TexStorage2D(gl::TEXTURE_2D, guess_mipmap_levels(width.max(height)), ifd.0, width, height);
                if !ptr.is_null() {
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width, height, ifd.1, ifd.2, ptr)
                }
            }

            // Generate mipmaps
            generate_mipmaps(gl::TEXTURE_2D, &self.params);

            // Generate filters
            generate_filters(gl::TEXTURE_2D, &self.params);
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            let bytes = self.params.bytes.as_loaded_mut().unwrap();
            bytes.clear();
            bytes.shrink_to(0);
        }

        // Add the texture to the pipeline
        pipeline.textures.insert(self)
    }

    fn find<'a>(pipeline: &'a crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a Self> {
        pipeline.textures.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut crate::pipeline::Pipeline, handle: &crate::pipeline::Handle<Self>) -> Option<&'a mut Self> {
        pipeline.textures.get_mut(handle)
    }

    fn disposed(self) {}
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
