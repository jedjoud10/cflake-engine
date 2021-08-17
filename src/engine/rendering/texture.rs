use crate::engine::core::cacher::CacheManager;
use crate::engine::resources::{LoadableResource, Resource};
use crate::engine::{resources::ResourceManager};
use bitflags::bitflags;
use gl;
use image::EncodableLayout;

use std::{
    ffi::{c_void},
    ptr::null,
};

bitflags! {
    pub struct TextureFlags: u8 {
        const MUTABLE = 0b00000001;
        const MIPMAPS = 0b00000010;
    }
}

// Texture filters
#[derive(Debug)]
pub enum TextureFilter {
	Linear,
	Nearest,
}

// Texture wrapping filters
#[derive(Debug)]
pub enum TextureWrapping {
	ClampToEdge,
	ClampToBorder,
	Repeat,
	MirroredRepeat,
}

// A texture
#[derive(Debug)]
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
	pub texture_filter: TextureFilter,
	pub texture_wrap_mode: TextureWrapping
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
            flags: TextureFlags::MUTABLE,
			samples: 0,
			texture_filter: TextureFilter::Linear,
			texture_wrap_mode: TextureWrapping::Repeat,
        }
    }
    // Cache the current texture and return it's reference
    pub fn cache_texture<'a>(
        self,
        texture_cacher: &'a mut CacheManager<Texture>,
    ) -> Option<(&'a Self, u16)> {
        let texture_name = self.name.clone();
        let texture_id = texture_cacher.cache_object(self, texture_name.as_str());
        return Some((texture_cacher.get_object(texture_name.as_str()).unwrap(), texture_id));
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
            true => self.flags |= TextureFlags::MUTABLE,
            false => self.flags &= !TextureFlags::MUTABLE,
        }
        self
    }
    // Set the generation of mipmaps
    pub fn enable_mipmaps(mut self) -> Self {
        self.flags |= TextureFlags::MIPMAPS;
        self
    }
    // Update the size of a current immutable texture
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
    // Load a texture from a file and auto caches it. Returns the cached texture and the cached ID
    pub fn load_texture<'a>(
        self,
        local_path: &str,
        resource_manager: &mut ResourceManager,
        texture_cacher: &'a mut CacheManager<Texture>,
    ) -> Option<(&'a Self, u16)> {
        // Load the resource
        let resource = resource_manager.load_packed_resource(local_path)?;
        // If the texture was already cached, just loaded from cache
        if texture_cacher.is_cached(local_path) {
            // It is indeed cached
            let texture = texture_cacher.get_object(local_path).unwrap();
			let texture_id = texture_cacher.get_object_id(local_path).unwrap();
            Some((texture, texture_id))
        } else {
            // If it not cached, then load the texture from that resource
            let texture = self
                .from_resource(resource)
                .cache_texture(texture_cacher)
                .unwrap();
            Some(texture)
        }
    }
	// Set the mag and min filters
	pub fn set_filter(mut self, filter: TextureFilter) -> Self {
		self.texture_filter = filter;
		self
	}
	// Set the wrapping mode
	pub fn set_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
		self.texture_wrap_mode = wrapping_mode;
		self
	}
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }
		
		if self.flags.contains(TextureFlags::MUTABLE) {
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
                // Set the texture parameters for a normal texture
				match self.texture_filter {
        			TextureFilter::Linear => {
						// 'Linear' filter
						gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
						gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
					},
       				TextureFilter::Nearest => {
						// 'Nearest' filter
						gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
						gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
					},
    			}							
            }

			// The texture is already bound to the TEXTURE_2D
            if self.flags.contains(TextureFlags::MIPMAPS) {
                // Create the mipmaps
                unsafe {
                    gl::GenerateMipmap(gl::TEXTURE_2D);
					// Set the texture parameters for a mipmapped texture
                    match self.texture_filter {
						TextureFilter::Linear => {
							// 'Linear' filter
							gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
							gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
						},
						   TextureFilter::Nearest => {
							// 'Nearest' filter
							gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
							gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
						},
					}
                }
            }
        } else {
            // Nobody loves you, OpenGL storage textures
            if self.flags.contains(TextureFlags::MIPMAPS) {
                // Create the mipmaps
                unsafe {
                    //gl::GenerateMipmap(gl::TEXTURE_2D);
                }
            }
        }

		// Set the wrap mode for the texture (Mipmapped or not)
		let wrapping_mode: i32;
		match self.texture_wrap_mode {
			TextureWrapping::ClampToEdge => wrapping_mode = gl::CLAMP_TO_EDGE as i32,
			TextureWrapping::ClampToBorder => wrapping_mode = gl::CLAMP_TO_BORDER as i32,
			TextureWrapping::Repeat => wrapping_mode = gl::REPEAT as i32,
			TextureWrapping::MirroredRepeat => wrapping_mode = gl::MIRRORED_REPEAT as i32,
		}
		unsafe {
			// Now set the actual wrapping mode in the opengl texture
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapping_mode);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapping_mode);
		}
			
		println!("{:?}", self);
        self
    }
}
