use std::ffi::c_void;

use super::Texture2D;
use crate::{
    basics::texture::{configurate, guess_mipmap_levels, RawTexture, Texture, TextureBytes, TextureFlags, TextureParams},
    object::ObjectSealed,
    pipeline::Pipeline,
};
use getset::{CopyGetters, Getters};

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
    dimensions: vek::Extent3<u32>,
}

impl Texture for BundledTexture2D {
    type Dimensions = vek::Extent3<u32>;

    fn storage(&self) -> Option<&RawTexture> {
        self.raw.as_ref()
    }
    fn params(&self) -> &TextureParams {
        &self.params
    }
    fn count_texels(&self) -> usize {
        self.dimensions.as_::<usize>().product()
    }
    fn dimensions(&self) -> vek::Extent3<u32> {
        self.dimensions
    }
    fn bytes(&self) -> &TextureBytes {
        &self.bytes
    }
}

impl ObjectSealed for BundledTexture2D {
    fn init(&mut self, _pipeline: &mut Pipeline) {
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
                    let levels = guess_mipmap_levels(self.dimensions.reduce_max()).max(1) as i32;
                    gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, levels, ifd.0, width, height, layers);
                    if !ptr.is_null() {
                        gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 0, width, height, layers, ifd.1, ifd.2, ptr);
                    }
                }
            }

            // Generate the OpenGL config for this texture
            configurate(gl::TEXTURE_2D_ARRAY, &self.params);
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            self.bytes.clear();
        }
    }

    fn disposed(self) {}
}

// Bundle multiple textures into a single bundled texture
pub fn bundle(textures: &[Texture2D]) -> Option<BundledTexture2D> {
    // We get the main dimensions from the first texture
    let first = textures.get(0)?;
    let (width, height) = (first.dimensions().w, first.dimensions().h);

    // Load the bytes
    let mut bytes: Vec<u8> = Vec::with_capacity(textures[0].count_bytes());
    for texture in textures.iter() {
        // Check if we have the same settings
        let d = texture.dimensions();
        if d.w != width || d.h != height {
            return None;
        }
        let texbytes = texture.bytes().as_valid().unwrap().iter();
        bytes.extend(texbytes);
    }

    Some(BundledTexture2D {
        raw: None,
        bytes: TextureBytes::Valid(bytes),
        params: *first.params(),
        dimensions: vek::Extent3::new(width, height, textures.len() as u32),
    })
}
