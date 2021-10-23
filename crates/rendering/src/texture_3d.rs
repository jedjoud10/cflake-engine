use std::ptr::null;

use others::CacheManager;

use super::{Texture, TextureDimensionType, TextureFilter, TextureWrapping};

// A 3D texture
#[derive(Debug)]
pub struct Texture3D {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub internal_texture: Texture,
}

impl Default for Texture3D {
    fn default() -> Self {
        Texture3D::new()
    }
}

// Impl of a Texture3D
impl Texture3D {
    // Just a new texture that can be tweaked before we load/generate it
    pub fn new() -> Self {
        Self {
            height: 1,
            width: 1,
            depth: 1,
            internal_texture: Texture::default(),
        }
    }

    
    
}

// Implement the wrapper stuff
impl Texture3D {
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
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        let t = self
            .internal_texture
            .generate_texture(bytes, TextureDimensionType::D3D(self.width, self.height, self.depth));
        self.internal_texture = t;
        return self;
    }
    // Cache
    pub fn cache(mut self, cacher: &mut CacheManager<Texture>) -> usize {
        let t = self.internal_texture;
        let name = t.name.clone();

        if name.trim().is_empty() {
            // If name is empty, create an unnamed object
            cacher.cache_unnamed_object(t)
        } else {
            cacher.cache_object(t, &name)
        }
    }
}
