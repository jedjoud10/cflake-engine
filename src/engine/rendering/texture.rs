use crate::engine::core::cacher::CacheManager;
use crate::engine::resources::{LoadableResource, Resource};
use crate::engine::{core::world::World, resources::ResourceManager};
use bitflags::bitflags;
use gl;
use image::EncodableLayout;
use std::io::Read;
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
};

bitflags! {
    pub struct TextureFlags: u8 {
        const Mutable = 0b00000001;
        const MipMaps = 0b00000010;
		const Multisampled = 0b00000100;
    }
}

// A texture
#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u16,
    pub height: u16,
    pub id: u32,
    pub name: String,
    pub internal_format: u32,
    pub format: u32,
    pub data_type: u32,
    pub flags: TextureFlags,
	pub samples: u8,
}

impl Default for Texture {
    fn default() -> Self {
        Texture::new()
    }
}

// Loadable resource
impl LoadableResource for Texture {
    // Load da texture resource
    fn from_resource(self, resource: &Resource) -> Self {
        match resource {
            Resource::Texture(texture, texture_name) => {
                let width = texture.width;
                let height = texture.height;

                // Turn the compressed png bytes into their raw form
                let mut image =
                    image::io::Reader::new(std::io::Cursor::new(&texture.compressed_bytes));
                image.set_format(image::ImageFormat::Png);
                let decoded = image.with_guessed_format().unwrap().decode().unwrap();
                // Read the image as a 32 bit image
                let rgba8_image = decoded.to_rgba8();

                // Set the proper dimensions and generate the texture from the resource's bytes
                let mut texture = self.set_dimensions(width, height).clone();
                // Set the texture name since the texture has an empty name
                texture.name = texture_name.clone();
                let mut new_texture = texture.generate_texture(rgba8_image.as_bytes().to_vec());
                return new_texture;
            }
            _ => {
                panic!("");
            }
        }
    }
}

// La texture
impl Texture {
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
            flags: TextureFlags::Mutable,
			samples: 0
        }
    }
    // Cache the current texture and return it's reference
    pub fn cache_texture<'a>(
        self,
        texture_cacher: &'a mut CacheManager<Texture>,
    ) -> Option<&'a Self> {
        let texture_name = self.name.clone();
        texture_cacher.cache_object(self, texture_name.as_str());
        return Some(texture_cacher.get_object(texture_name.as_str()).ok()?);
    }
    // Set the height and width of the soon to be generated texture
    pub fn set_dimensions(mut self, width: u16, height: u16) -> Self {
        self.height = height;
        self.width = width;
        self
    }
    // The internal format and data type of the soon to be generated texture
    pub fn set_idf(mut self, internal_format: u32, format: u32, data_type: u32) -> Self {
        self.internal_format = internal_format;
        self.format = format;
        self.data_type = data_type;
        self
    }
    // Set if we should use the new opengl api (Gl tex storage that allows for immutable texture) or the old one
    pub fn set_mutable(mut self, mutable: bool) -> Self {
        match mutable {
            true => self.flags |= TextureFlags::Mutable,
            false => self.flags = self.flags & !TextureFlags::Mutable,
        }
        self
    }
    // Set the generation of mipmaps
    pub fn enable_mipmaps(mut self) -> Self {
        self.flags |= TextureFlags::MipMaps;
        self
    }
	// Make this texture a multisampled texture (Only used for the texture attachements of the framebuffer)
	pub fn enable_multisampling(mut self, samples: u8) -> Self {
		self.flags |= TextureFlags::Multisampled;
		self.samples = samples;
		self
	}
    // Update the size of a current immutable texture
    pub fn update_size(&self, width: u16, height: u16) {
        if self.flags.contains(TextureFlags::Mutable) {
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
    // Load a texture from a file and auto caches it. Returns the cached ID of the texture
    pub fn load_texture<'a>(
        mut self,
        local_path: &str,
        resource_manager: &mut ResourceManager,
        texture_cacher: &'a mut CacheManager<Texture>,
    ) -> Option<&'a Self> {
        // Load the resource
        let resource = resource_manager.load_packed_resource(local_path)?;
        // If the texture was already cached, just loaded from cache
        if texture_cacher.is_cached(local_path) {
            // It is indeed cached
            let texture = texture_cacher.get_object(local_path).unwrap();
            return Some(texture);
        } else {
            // If it not cached, then load the texture from that resource
            let texture = self
                .from_resource(resource)
                .cache_texture(texture_cacher)
                .unwrap();
            return Some(texture);
        }
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        
        let mut pointer: *const c_void = null();
        if bytes.len() > 0 {
            pointer = bytes.as_ptr() as *const c_void;
        }
		
		if self.flags.contains(TextureFlags::Multisampled) {
			// This is a multisampled texture
			unsafe {
                gl::GenTextures(1, &mut self.id as *mut u32);
                gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.id);
                gl::TexImage2DMultisample(
                    gl::TEXTURE_2D_MULTISAMPLE,
                    self.samples as i32,
                    self.internal_format as u32,
                    self.width as i32,
                    self.height as i32,
                    gl::TRUE,
                );
            }
		} else if self.flags.contains(TextureFlags::Mutable) {
            // It's a normal mutable texture
            unsafe {
                gl::GenTextures(1, &mut self.id as *mut u32);
                gl::BindTexture(gl::TEXTURE_2D, self.id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    self.internal_format as i32,
                    self.width as i32,
                    self.height as i32,
                    0,
                    self.format,
                    self.data_type,
                    pointer,
                );
                // Mag and min filters
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

			// The texture is already bound to the TEXTURE_2D
            if self.flags.contains(TextureFlags::MipMaps) {
                // Create the mipmaps
                unsafe {
                    gl::GenerateMipmap(gl::TEXTURE_2D);
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MIN_FILTER,
                        gl::LINEAR_MIPMAP_LINEAR as i32,
                    );
                }
            }
        } else {
            // Nobody loves you, OpenGL storage textures
            if self.flags.contains(TextureFlags::MipMaps) {
                // Create the mipmaps
                unsafe {
                    //gl::GenerateMipmap(gl::TEXTURE_2D);
                }
            }
        }
		println!("{:?}", self);
        return self;
    }
}
