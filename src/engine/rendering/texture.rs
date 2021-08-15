use crate::engine::core::cacher::CacheManager;
use crate::engine::resources::Resource;
use crate::engine::{core::world::World, resources::ResourceManager};
use gl;
use image::EncodableLayout;
use bitflags::bitflags;
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
	}
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
}

impl Default for Texture {
	fn default() -> Self {
		Texture::new()
	}
}

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
		}
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
		self
		/*
		match mutable {
    		true => self.flags |= TextureFlags::Mutable,
    		false => self.flags = self.flags & !TextureFlags::Mutable,
		}
		*/
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
	// Load a texture from a resource and auto caches it. Returns the cached ID of the texture
	pub fn load_resource<'a>(mut self, resource: &Resource, texture_cacher: &'a mut CacheManager<Texture>) -> Option<&'a Self> {
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
				self = self.set_dimensions(width, height);
                let mut new_texture = self.generate_texture(rgba8_image.as_bytes().to_vec());
				// Set the texture name since the texture has an empty name
				new_texture.name = texture_name.clone();
				// Cache the texture for later use
				texture_cacher.cache_object(new_texture, &texture_name);
                return Some(texture_cacher.get_object(texture_name).unwrap());
            }
            _ => return None,
        }
	}
	// Load a texture from a file and auto caches it. Returns the cached ID of the texture
	pub fn load_texture<'a>(mut self, local_path: &str, resource_manager: &mut ResourceManager, texture_cacher: &'a mut CacheManager<Texture>) -> Option<&'a Self> {
		// Load the resource
		let resource = resource_manager.load_packed_resource(local_path)?;
		// Then load the texture from that resource
		let texture = self.load_resource(resource, texture_cacher)?;
		return Some(texture);		
	}
	// Generate an empty texture, could either be a mutable one or an immutable one
	pub fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
		println!("{:?}", self);
		if self.flags.contains(TextureFlags::Mutable) {
			// Is mutable texture
			unsafe {
				gl::GenTextures(1, &mut self.id as *mut u32);
				gl::BindTexture(gl::TEXTURE_2D, self.id);
				let mut pointer: *const c_void = null();
				if bytes.len() > 0 {
					pointer = bytes.as_ptr() as *const c_void;
				} 
				gl::TexImage2D(
					gl::TEXTURE_2D,
					0,
					self.internal_format as i32,
					self.width as i32,
					self.height as i32,
					0,
					self.format,
					self.data_type,
					pointer
				);
				// Mag and min filters
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			}
		} else {
			// Is immutable texture
			unsafe {
				gl::GenTextures(1, &mut self.id as *mut u32);
				gl::BindTexture(gl::TEXTURE_2D, self.id);
				gl::TexStorage2D(
					gl::TEXTURE_2D,
					0,
					self.internal_format,
					self.width as i32,
					self.height as i32
				);
				gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, self.width as i32, self.height as i32, self.format, self.data_type, bytes.as_ptr() as *const c_void);
				// Mag and min filters
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			}

			if self.flags.contains(TextureFlags::MipMaps) {
				// Create the mipmaps
				unsafe {
					gl::GenerateMipmap(gl::TEXTURE_2D);
				}
			}
		}
		return self;
	}
}
