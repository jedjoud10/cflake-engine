use getset::{Getters, CopyGetters};
use gl::types::{GLuint, GLint};
use crate::{basics::texture::{TextureLayout, TextureWrapMode, TextureFlags, TextureFilter, TextureParams, Texture, get_texel_byte_size, TextureBytes, ResizableTexture, TextureStorage}, object::PipelineElement, pipeline::{Pipeline, Handle}};
use super::{Texture2D, TextureBuilder};

// A combination of multiple texture2D's
// This represents an OpenGL Array Texture
#[derive(Getters, CopyGetters)]
pub struct BundledTexture2D {
    // Storage
    storage: Option<TextureStorage>,

    // Params
    params: TextureParams,

    // Texture dimensions
    dimensions: vek::Vec3<u16>,
}

impl Texture for BundledTexture2D {
    type Dimensions = vek::Vec3<u16>;

    fn target(&self) -> GLuint {
        self.storage.as_ref().expect("OpenGL BundledTexture2D is invalid!").target()
    }
    fn texture(&self) -> GLuint {
        self.storage.as_ref().expect("OpenGL BundledTexture2D is invalid!").name()
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
}

impl PipelineElement for BundledTexture2D {
    fn add(self, pipeline: &mut Pipeline) -> Handle<Self> {
        pipeline.bundled_textures.insert(self)
    }

    fn find<'a>(pipeline: &'a Pipeline, handle: &Handle<Self>) -> Option<&'a Self> {
        pipeline.bundled_textures.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut Pipeline, handle: &Handle<Self>) -> Option<&'a mut Self> {
        pipeline.bundled_textures.get_mut(handle)
    }

    fn disposed(self) {
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
    pub fn build(self, params: Option<TextureParams>) -> Option<BundledTexture2D> {
        // We get the main dimensions from the first texture
        let first = self.textures.get(0)?;
        let (width, height) = (first.dimensions().x, first.dimensions().y);

        // Load the bytes
        let mut bytes: Vec<u8> = Vec::with_capacity(self.textures[0].count_bytes());
        for texture in self.textures.iter() {
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
            storage: None,
            params: TextureParams {
                bytes: TextureBytes::Loaded(bytes),
                layout: params.layout,
                filter: params.filter,
                wrap: params.wrap,
                flags: params.flags,
            },
            dimensions: vek::Vec3::new(width, height, self.textures.len() as u16),
        })
    }
}
