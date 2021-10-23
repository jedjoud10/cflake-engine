use std::ptr::null;

use super::{Texture, TextureDimensionType, TextureFilter, TextureWrapping};
use errors::ResourceError;
use image::EncodableLayout;
use others::CacheManager;
use resources::{LoadableResource, Resource, ResourceManager};

// A 2D texture
#[derive(Debug)]
pub struct Texture2D {
    pub width: u16,
    pub height: u16,
    pub internal_texture: Texture,
}

impl Default for Texture2D {
    fn default() -> Self {
        Texture2D::new()
    }
}



// Loading / caching stuff for Texture2D
impl Texture2D {
    
}

// Impl of a texture2D
impl Texture2D {
    // Just a new texture that can be tweaked before we load/generate it
    pub fn new() -> Self {
        Self {
            height: 1,
            width: 1,
            internal_texture: Texture::default(),
        }
    }

    // Set the height and width of the soon to be generated texture
    pub fn set_dimensions(mut self, width: u16, height: u16) -> Self {
        self.height = height;
        self.width = width;
        self.internal_texture.dimension_type = TextureDimensionType::D2D(width, height);
        self
    }
    // Set the height and width of the soon to be generated texture using a vector
    pub fn set_dimensions_vec2(mut self, dimensions: veclib::Vector2<u16>) -> Self {
        self.height = dimensions.y;
        self.width = dimensions.x;
        self.internal_texture.dimension_type = TextureDimensionType::D2D(self.width, self.height);
        self
    }
    // Update the size of the current texture
    pub fn update_size(&mut self, width: u16, height: u16) {
        // This is a normal texture getting resized
        
    }
}

// Implement the wrapper stuff
impl Texture2D {
    // The internal format and data type of the soon to be generated texture
    pub fn set_idf(mut self, internal_format: u32, format: u32, data_type: u32) -> Self {
        let t = self.internal_texture.set_idf(internal_format, format, data_type);
        self.internal_texture = t;
        return self;
    }
    // Set if we should use the new opengl api (Gl tex storage that allows for immutable texture) or the old one
    pub fn set_mutable(mut self, mutable: bool) -> Self {
        let t = self.internal_texture.set_mutable(mutable);
        self.internal_texture = t;
        return self;
    }
    // Set the generation of mipmaps
    pub fn enable_mipmaps(mut self) -> Self {
        let t = self.internal_texture.enable_mipmaps();
        self.internal_texture = t;
        return self;
    }
    // Set the mag and min filters
    pub fn set_filter(mut self, filter: TextureFilter) -> Self {
        let t = self.internal_texture.set_filter(filter);
        self.internal_texture = t;
        return self;
    }
    // Set the wrapping mode
    pub fn set_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        let t = self.internal_texture.set_wrapping_mode(wrapping_mode);
        self.internal_texture = t;
        return self;
    }
    // Set the internal's texture name
    pub fn set_internal_name(mut self, name: &str) -> Self {
        self.internal_texture.name = name.to_string();
        self
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        let dimension_type = self.internal_texture.dimension_type.clone();
        let t = self.internal_texture.generate_texture(bytes, dimension_type);
        self.internal_texture = t;
        return self;
    }
}

// The texture 2D can be instanced
impl others::Instance for Texture2D {
    fn set_name(&mut self, string: String) {
        self.internal_texture.name = string
    }
    fn get_name(&self) -> String {
        self.internal_texture.name.clone()
    }
}
