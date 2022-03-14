use std::ffi::c_void;

use super::{Texture2D, TextureBuilder};
use crate::{
    basics::texture::{
        generate_filters, generate_mipmaps, get_texel_byte_size, guess_mipmap_levels, verify_byte_size, RawTexture, ResizableTexture, Texture, TextureBytes, TextureFilter,
        TextureFlags, TextureLayout, TextureParams, TextureWrapMode,
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

    // The texture bytes
    bytes: TextureBytes,

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
    fn bytes(&self) -> &TextureBytes {
        &self.bytes
    }
}

impl PipelineElement for BundledTexture2D {
    fn add(mut self, pipeline: &mut Pipeline) -> Handle<Self> {
        // Create the raw texture array wrapper
        let texture = unsafe { RawTexture::new(gl::TEXTURE_2D_ARRAY, &self.params) };
        let ifd = texture.ifd;
        self.raw = Some(texture);
        let (width, height, layers) = self.dimensions.as_::<i32>().into_tuple();
        // Texture generation, SRGB, mipmap, filters
        unsafe {
            // Don't allocate anything if the textures dimensions are invalid
            if width != 0 && height != 0 {
                let ptr = self.bytes.get_ptr() as *const c_void;
                // Depends if it is resizable or not
                if self.params.flags.contains(TextureFlags::RESIZABLE) {
                    // Dynamic
                    gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, ifd.0 as i32, width, height, layers, 0, ifd.1, ifd.2, ptr);
                } else {
                    // Static
                    let levels = guess_mipmap_levels(width.max(height).max(layers)).max(1);
                    gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, levels, ifd.0, width, height, layers);
                    if !ptr.is_null() {
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 0, width, height, layers, ifd.1, ifd.2, ptr);
                    }
                }
            }

            // Generate mipmaps
            generate_mipmaps(gl::TEXTURE_2D_ARRAY, &self.params);

            // Generate filters
            generate_filters(gl::TEXTURE_2D_ARRAY, &self.params);
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            self.bytes.clear();
        }
        // Add the bundled texture to the pipeline
        pipeline.bundled_textures.insert(self)
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
            let texbytes = texture.bytes().as_valid().unwrap().iter();
            bytes.extend(texbytes);
        }

        // Use the first texture's params, in case we don't have an override
        let params = params.as_ref().unwrap_or(first.params());
        Some(BundledTexture2D {
            raw: None,
            bytes: TextureBytes::Valid(bytes),
            params: TextureParams {
                layout: params.layout,
                filter: params.filter,
                wrap: params.wrap,
                flags: params.flags,
                custom: params.custom.clone(),
            },
            dimensions: vek::Vec3::new(width, height, textures.len() as u16),
        })
    }
}
