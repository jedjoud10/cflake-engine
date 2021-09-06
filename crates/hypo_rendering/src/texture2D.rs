use std::ptr::null;

use super::{TextureWrapping, TextureFilter, TextureFlags, Texture};
use hypo_resources::{LoadableResource, Resource, ResourceManager};
use hypo_others::CacheManager;

// A 2D texture
#[derive(Debug)]
pub struct Texture2D {
    pub width: u16,
    pub height: u16,
    pub id: u32,
    pub name: String,
    pub internal_format: u32,
    pub format: u32,
    pub data_type: u32,
    pub flags: TextureFlags,
    pub samples: u8,
    pub texture_filter: TextureFilter,
    pub texture_wrap_mode: TextureWrapping,
}

impl Default for Texture2D {
    fn default() -> Self {
        Texture2D::new()
    }
}

// Loadable resource
impl LoadableResource for Texture2D {
    // Load da texture resource
    fn from_resource(self, resource: &Resource) -> Self {
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
                let mut texture = self.set_dimensions(width, height);
                // Set the texture name since the texture has an empty name
                texture.name = texture_name.clone();
                let new_texture = texture.generate_texture(rgba8_image.as_bytes().to_vec());
                new_texture
            }
            _ => {
                panic!("");
            }
        }
    }
}

// Loading / caching stuff for Texture2D
impl Texture2D {
    // Cache the current texture and return it's reference
    pub fn cache_texture<'a>(self, texture_cacher: &'a mut CacheManager<Texture2D>) -> Option<(&'a Self, u16)> {
        let texture_name = self.name.clone();
        let texture_id = texture_cacher.cache_object(self, texture_name.as_str());
        return Some((texture_cacher.get_object(texture_name.as_str()).unwrap(), texture_id));
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
            let texture = self.from_resource(resource).cache_texture(texture_cacher).unwrap();
            Ok(texture)
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
            id: 0,
            name: String::new(),
            internal_format: gl::RGBA,
            format: gl::RGBA,
            data_type: gl::UNSIGNED_BYTE,
            flags: TextureFlags::MUTABLE,
            samples: 0,
            texture_filter: TextureFilter::Linear,
            texture_wrap_mode: TextureWrapping::Repeat,
        }
    }
    
    // Set the height and width of the soon to be generated texture
    pub fn set_dimensions(mut self, width: u16, height: u16) -> Self {
        self.height = height;
        self.width = width;
        self
    }
    // Update the size of the current texture
    pub fn update_size(&self, width: u16, height: u16) {
        // This is a normal texture getting resized
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.internal_format as i32,
                width as i32,
                height as i32,
                0,
                self.format,
                self.data_type,
                null(),
            );
        }
    }     
}

