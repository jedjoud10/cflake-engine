use std::{ffi::c_void, mem::ManuallyDrop};

use assets::Asset;
use getset::{CopyGetters, Getters};

use crate::{
    basics::texture::{
        apply_customs, generate_filters, generate_mipmaps, guess_mipmap_levels, verify_byte_size, RawTexture, ResizableTexture, Texture, TextureBytes, TextureFlags, TextureParams,
    },
    object::PipelineElement,
};

// A simple two dimensional OpenGL texture
#[derive(Default, Getters, CopyGetters)]
pub struct Texture2D {
    // Storage
    raw: Option<RawTexture>,

    // The texture bytes
    bytes: TextureBytes,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Extent2<u16>,
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
    pub fn bytes(mut self, bytes: Vec<u8>) -> Self {
        self.inner.bytes = TextureBytes::Valid(bytes);
        self
    }
    pub fn dimensions(mut self, dimensions: vek::Extent2<u16>) -> Self {
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
                    let levels = guess_mipmap_levels(width.max(height)).max(1);
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

            // Apply customs
            apply_customs(gl::TEXTURE_2D, &self.params);
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            self.bytes.clear();
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

impl Texture for Texture2D {
    type Dimensions = vek::Extent2<u16>;
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
    fn resize_then_write(&mut self, dimensions: vek::Extent2<u16>, bytes: Vec<u8>) -> Option<()> {
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
    fn deserialize(self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let i = std::time::Instant::now();
        let image = image::load_from_memory(bytes).unwrap();
        let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
        // Flip
        let image = image.flipv();
        let (width, height) = (image.width() as u16, image.height() as u16);
        let bytes = image.into_bytes();
        assert!(!bytes.is_empty(), "Cannot load in an empty texture!");
        println!("Took {}ms to load texture", i.elapsed().as_millis());
        Some(TextureBuilder::default().dimensions(vek::Extent2::new(width, height)).bytes(bytes).build())
    }
}
