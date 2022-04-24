use std::{ffi::c_void, mem::ManuallyDrop};

use assets::Asset;
use getset::{CopyGetters, Getters};
use image::{imageops::FilterType, DynamicImage};

use crate::{
    basics::texture::{
        generate_filters, generate_mipmaps, get_texel_byte_size, guess_mipmap_levels, verify_byte_size, RawTexture, ResizableTexture, Texture, TextureBytes, TextureFlags,
        TextureParams, TextureLayout,
    },
    object::ObjectSealed,
};

// A simple two dimensional OpenGL texture
#[derive(Getters, CopyGetters)]
pub struct Texture2D {
    // Storage
    raw: Option<RawTexture>,

    // The texture bytes
    bytes: TextureBytes,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Extent2<u32>,
}

impl Texture2D {
    // Create a texture with the specified dimensions, parameters, and bytes
    pub fn new(dimensions: vek::Extent2<u32>, bytes: Option<Vec<u8>>, params: TextureParams) -> Self {
        // Checking byte counts
        if let Some(bytes) = &bytes {
            let bytes_per_texel = get_texel_byte_size(params.layout.internal_format);
            assert_eq!(dimensions.product() as usize, bytes.len() / bytes_per_texel, "Number of bytes invalid");
        }

        Self {
            raw: None,
            bytes: TextureBytes::Valid(bytes.unwrap_or_default()),
            params,
            dimensions,
        }
    }
}

impl ObjectSealed for Texture2D {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        // TODO: Fix code duplication between bundledtexture2d and texture2d and cubemap
        // Create the raw texture wrapper
        let texture = unsafe { RawTexture::new(gl::TEXTURE_2D, &self.params) };
        let ifd = texture.ifd;
        self.raw = Some(texture);
        let (width, height) = self.dimensions.as_::<i32>().into_tuple();
        // Texture generation, SRGB, mipmap, filters
        unsafe {
            // Don't allocate anything if the textures dimensions are invalid
            if width != 0 && height != 0 {
                let ptr = self.bytes.get_ptr() as *const c_void;
                // Depends if it is resizable or not
                if self.params.flags.contains(TextureFlags::RESIZABLE) {
                    // Dynamic
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0 as i32, width, height, 0, ifd.1, ifd.2, ptr);
                } else {
                    // Static
                    let levels = guess_mipmap_levels(self.dimensions.reduce_max()).max(1) as i32;
                    gl::TexStorage2D(gl::TEXTURE_2D, levels, ifd.0, width, height);
                    if !ptr.is_null() {
                        gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width, height, ifd.1, ifd.2, ptr);
                    }
                }
            }

            // Generate mipmaps
            generate_mipmaps(gl::TEXTURE_2D, &self.params);

            // Generate filters
            generate_filters(gl::TEXTURE_2D, &self.params);
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            self.bytes.clear();
        }
    }
}

impl Texture for Texture2D {
    type Dimensions = vek::Extent2<u32>;
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
    fn bytes(&self) -> &TextureBytes {
        &self.bytes
    }
}

impl ResizableTexture for Texture2D {
    fn resize_then_write(&mut self, dimensions: vek::Extent2<u32>, bytes: Vec<u8>) -> Option<()> {
        // Check if we can even resize the texture
        if !self.params.flags.contains(TextureFlags::RESIZABLE) {
            return None;
        }

        // Resize the texture
        self.dimensions = dimensions;
        let (width, height) = dimensions.as_::<i32>().into_tuple();
        // This will also automatically clear the image
        let raw = self.raw.as_ref();
        if let Some(raw) = raw {
            unsafe {
                // Manually drop the vector when we are done with it
                let manual = ManuallyDrop::new(bytes);
                let ptr = verify_byte_size(self.count_bytes(), manual.as_ref())?;
                let ifd = raw.ifd;
                gl::BindTexture(gl::TEXTURE_2D, raw.name);
                gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0 as i32, width, height, 0, ifd.1, ifd.2, ptr);

                // Drop it (save the bytes if possible)
                if self.params.flags.contains(TextureFlags::PERSISTENT) {
                    self.bytes = TextureBytes::Valid(ManuallyDrop::into_inner(manual));
                }
            }
        }
        Some(())
    }
}

// Load a Texture2D
impl Asset for Texture2D {
    type OptArgs = TextureParams;
    fn deserialize(_meta: &assets::metadata::AssetMetadata, bytes: &[u8], input: Self::OptArgs) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes (switch on the texture layout mode)
        let image = image::load_from_memory(bytes).unwrap();

        let image = if input.layout == TextureLayout::HDR {
            println!("A");
            image::DynamicImage::ImageRgba32F(image.into_rgba32f())
        } else {
            println!("A");
            image::DynamicImage::ImageRgba8(image.into_rgba8())
        };
        
        println!("B");
        // Flip the image and fetch it's bytes
        let (w, h) = (image.width(), image.height());
        //let image = image.flipv();
        println!("{}", image.as_flat_samples_f32().is_some());
        let bytes = image.into_bytes();
        println!("C");
        
        // "Oh no..." check
        assert!(!bytes.is_empty(), "Cannot load in an empty texture!");

        // Loaded engine texture, simply return it
        Some(Texture2D {
            raw: None,
            bytes: TextureBytes::Valid(bytes),
            dimensions: vek::Extent2::new(w, h),
            params: input,
        })
    }
}
