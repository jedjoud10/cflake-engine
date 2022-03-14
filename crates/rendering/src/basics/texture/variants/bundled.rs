use super::{Texture2D, TextureBuilder};
use crate::{
    basics::texture::{
        get_texel_byte_size, store_bytes, verify_byte_size, RawTexture, ResizableTexture, Texture, TextureBytes, TextureFilter, TextureFlags, TextureLayout, TextureParams,
        TextureWrapMode,
    },
    object::PipelineElement,
    pipeline::{Handle, Pipeline},
};
use getset::{CopyGetters, Getters};
use gl::types::{GLint, GLuint};

// A combination of multiple texture2D's
// This represents an OpenGL Array Texture
#[derive(Getters, CopyGetters)]
pub struct BundledTexture2D {
    // Storage
    raw: Option<RawTexture>,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Vec3<u16>,
}

impl Texture for BundledTexture2D {
    type Dimensions = vek::Vec3<u16>;

    fn storage(&self) -> Option<&RawTexture> {
        self.raw.as_ref()
    }
    fn params(&self) -> &TextureParams {
        &self.params
    }
    fn count_texels(&self) -> usize {
        self.dimensions.as_::<usize>().product()
    }
    fn dimensions(&self) -> vek::Vec3<u16> {
        self.dimensions
    }
    fn write(&mut self, bytes: Vec<u8>) -> Option<()> {
        // Write to the OpenGL texture first
        let ptr = verify_byte_size(self.count_bytes(), &bytes)?;

        // Write
        if let Some(raw) = self.raw.as_ref() {
            let (width, height, layers) = (self.dimensions).as_::<i32>().into_tuple();
            let ifd = raw.ifd;
            unsafe {
                gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 0, width, height, layers, ifd.1, ifd.2, ptr);
            }
        }

        // Then save the bytes if possible
        store_bytes(self.params().flags, bytes, &mut self.params.bytes);
        Some(())
    }
}

impl PipelineElement for BundledTexture2D {
    fn add(self, pipeline: &mut Pipeline) -> Handle<Self> {
        pipeline.bundled_textures.insert(self);
        todo!()
    }

    fn find<'a>(pipeline: &'a Pipeline, handle: &Handle<Self>) -> Option<&'a Self> {
        pipeline.bundled_textures.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut Pipeline, handle: &Handle<Self>) -> Option<&'a mut Self> {
        pipeline.bundled_textures.get_mut(handle)
    }

    fn disposed(self) {}
}

// A texture bundler that creates a 2D texture array from a set of textures
#[derive(Default)]
pub struct BundledTextureBuilder;

impl BundledTextureBuilder {
    // Build the bundled texture
    pub fn build(textures: &[Texture2D], params: Option<TextureParams>) -> Option<BundledTexture2D> {
        // We get the main dimensions from the first texture
        let first = textures.get(0)?;
        let (width, height) = (first.dimensions().x, first.dimensions().y);

        // Load the bytes
        let mut bytes: Vec<u8> = Vec::with_capacity(textures[0].count_bytes());
        for texture in textures.iter() {
            // Check if we have the same settings
            let d = texture.dimensions();
            if d.x != width || d.y != height {
                return None;
            }
            let texbytes = texture.params().bytes.as_loaded().unwrap().iter();
            bytes.extend(texbytes);
        }

        // Use the first texture's params, in case we don't have an override
        let params = params.as_ref().unwrap_or(first.params());
        Some(BundledTexture2D {
            raw: None,
            params: TextureParams {
                bytes: TextureBytes::Loaded(bytes),
                layout: params.layout,
                filter: params.filter,
                wrap: params.wrap,
                flags: params.flags,
            },
            dimensions: vek::Vec3::new(width, height, textures.len() as u16),
        })
    }
}
