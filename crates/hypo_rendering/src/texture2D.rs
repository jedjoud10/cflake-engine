use std::{ffi::c_void, ptr::null};

use super::{Texture, TextureDimensionType, TextureFilter, TextureFlags, TextureWrapping};
use hypo_others::CacheManager;
use hypo_resources::{LoadableResource, Resource, ResourceManager};
use image::EncodableLayout;

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

// Loadable resource
impl LoadableResource for Texture2D {
    // Load a texture 2D from a resource file
    fn from_resource(resource: &Resource) -> Option<Self> {
        match resource {
            Resource::Texture(texture, texture_name) => {
                let width = texture.width;
                let height = texture.height;

                // Turn the compressed png bytes into their raw form
                let mut image = image::io::Reader::new(std::io::Cursor::new(&texture.compressed_bytes));
                image.set_format(image::ImageFormat::Png);
                let decoded = image.with_guessed_format().unwrap().decode().unwrap();
                // Read the image as a 32 bit image
                let rgba8_image = decoded.to_rgba8();

                // Set the proper dimensions and generate the texture from the resource's bytes
                let mut texture = Self::new().set_dimensions(width, height);
                // Set the texture name since the texture has an empty name
                texture.internal_texture.name = texture_name.clone();
                let new_texture = texture
                    .internal_texture
                    .generate_texture(rgba8_image.as_bytes().to_vec(), TextureDimensionType::D_2D(width, height));
                texture.internal_texture = new_texture;
                Some(texture)
            }
            _ => { None }
        }
    }
}

// Loading / caching stuff for Texture2D
impl Texture2D {
    // Cache the current texture and return it's reference
    pub fn cache_texture<'a>(self, texture_cacher: &'a mut CacheManager<Texture2D>) -> Option<(&'a mut Self, u16)> {
        let texture_name = self.internal_texture.name.clone();
        let texture_id = texture_cacher.cache_object(self, texture_name.as_str());
        return Some((texture_cacher.get_object_mut(texture_name.as_str()).unwrap(), texture_id));
    }
    // Load a texture from a file and auto caches it. Returns the cached texture and the cached ID
    pub fn load_texture<'a>(
        self,
        local_path: &str,
        resource_manager: &mut ResourceManager,
        texture_cacher: &'a mut CacheManager<Texture2D>,
    ) -> Result<(&'a Self, u16), hypo_errors::ResourceError> {
        // Load the resource
        let resource = resource_manager.load_packed_resource(local_path)?;
        // If the texture was already cached, just loaded from cache
        if texture_cacher.is_cached(local_path) {
            // It is indeed cached
            let texture = texture_cacher.get_object(local_path).unwrap();
            let texture_id = texture_cacher.get_object_id(local_path).unwrap();
            Ok((texture, texture_id))
        } else {
            // If it not cached, then load the texture from that resource
            let (texture, texture_id) = Self::from_resource(resource).ok_or(hypo_errors::ResourceError::new_str("Could not load texture!"))?.cache_texture(texture_cacher).unwrap();
            // Copy the texture settings from "self" to "texture"
            texture.width = self.width;
            texture.height = self.height;
            texture.internal_texture.internal_format = self.internal_texture.internal_format;
            texture.internal_texture.format = self.internal_texture.format;
            texture.internal_texture.data_type = self.internal_texture.data_type;
            texture.internal_texture.flags = self.internal_texture.flags;
            texture.internal_texture.filter = self.internal_texture.filter;

            Ok((texture, texture_id))
        }
    }
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
        self.internal_texture.dimension_type = TextureDimensionType::D_2D(width, height);
        self
    }
    // Update the size of the current texture
    pub fn update_size(&self, width: u16, height: u16) {
        // This is a normal texture getting resized
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.internal_texture.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.internal_texture.internal_format as i32,
                width as i32,
                height as i32,
                0,
                self.internal_texture.format,
                self.internal_texture.data_type,
                null(),
            );
        }
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
    // Get the cached texture id
    pub fn get_cached_id(self) -> u16 {
        self.internal_texture.cached_texture_id
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        let t = self.internal_texture.generate_texture(bytes, TextureDimensionType::D_2D(self.width, self.height));
        self.internal_texture = t;
        return self;
    }
}
