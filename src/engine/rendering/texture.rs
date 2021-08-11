use crate::engine::resources::Resource;
use crate::engine::{core::world::World, resources::ResourceManager};
use gl;
use image::EncodableLayout;
use std::{
	collections::HashMap,
	ffi::{c_void, CString},
	mem::size_of,
	ptr::null,
};

// A texture manager
#[derive(Default)]
pub struct TextureManager {
	pub texture_ids: HashMap<String, u16>,
	pub cached_textures: Vec<Texture>,
}

impl TextureManager {
	// Get a reference to a texture from the texture manager's cache
	pub fn get_texture(&self, id: u16) -> &Texture {
		return self.cached_textures.get(id as usize).unwrap();
	}
	// Get a mutable reference to a texture from the texture manager's cache
	pub fn get_texture_mut(&mut self, id: i16) -> &mut Texture {
		return self.cached_textures.get_mut(id as usize).unwrap();
	}
	// Add a texture to the manager
	pub fn cache_texture(&mut self, texture: Texture) -> u16 {
		let name_clone = texture.name.clone();
		// Make sure the texture isn't cached already
		if !self.texture_ids.contains_key(&name_clone) {
			self.cached_textures.push(texture);
			let texture_id = self.cached_textures.len() as i16 - 1;
			println!(
				"Cache texture: '{}' with texture id '{}'",
				&name_clone, &texture_id
			);
			self.texture_ids.insert(name_clone, texture_id as u16);
			return texture_id as u16;
		} else {
			return self.texture_ids.get(&name_clone).unwrap().clone();
		}
	}
	// Get the texture id of a specific texture using it's name
	pub fn get_texture_id(&self, name: &str) -> u16 {
		// Check if the texture even exists
		if self.texture_ids.contains_key(&name.to_string()) {
			return self.texture_ids.get(name).unwrap().clone();
		} else {
			panic!("Texture was not cached!");
		}
	}
}

// A texture
#[derive(Default)]
pub struct Texture {
	pub width: u16,
	pub height: u16,
	pub id: u32,
	pub name: String,
	pub internal_format: u32,
	pub format: u32,
	pub data_type: u32,
}

impl Texture {
	// Loads a texture and caches it, then returns the texture id
	pub fn load_from_file(
		file: &str,
		resource_manager: &mut ResourceManager,
		texture_manager: &mut TextureManager,
	) -> Option<u16> {
		let mut id = 0_u16;
		// Check if the texture was cached
		
		if texture_manager.texture_ids.contains_key(file) {
			// Texture was already cached
			println!("Load cached texture '{}'", file);
			let cached_texture_id = texture_manager.get_texture_id(file);
			id = cached_texture_id;
			let cached_texture = texture_manager.get_texture(id);
		} else {
			// First time loading this texture
			let texture_resource = resource_manager.load_resource(file, "textures\\")?;
			let texture = Texture::from_resource(texture_resource)?;
			id = texture_manager.cache_texture(texture);
		}
		return Some(id);
	}
	// Convert the resource to a texture
	pub fn from_resource(resource: &Resource) -> Option<Self> {
		match resource {
			Resource::Texture(texture) => {
				let width = texture.width;
				let height = texture.height;

				// Turn the compressed png bytes into their raw form
				let mut image = image::io::Reader::new(std::io::Cursor::new(&texture.compressed_bytes));
				image.set_format(image::ImageFormat::Png);
				let decoded = image.with_guessed_format().unwrap().decode().unwrap();
				let test = decoded.to_rgba8();

				let mut new_texture = Self::create_rgba_texture(texture.name.clone(), width, height, &test.as_bytes().to_owned());
				new_texture.name = texture.name.clone();
				return Some(new_texture);
			}
			_ => return None,
		}
	}
	// Creates a new empty texture from a specified size
	pub fn create_new_texture(
		width: u16,
		height: u16,
		internal_format: u32,
		format: u32,
		data_type: u32,
	) -> Self {
		let mut texture = Self {
			width,
			height,
			id: 0,
			internal_format,
			name: String::from("Untitled"),
			format,
			data_type,
		};

		// Create the OpenGL texture and set it's data to null since it's empty
		unsafe {
			gl::GenTextures(1, &mut texture.id as *mut u32);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				texture.internal_format as i32,
				width as i32,
				height as i32,
				0,
				texture.format,
				texture.data_type,
				null(),
			);

			// Mag and min filters
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		}

		return texture;
	}
	// Creates a rgb texture from a vector filled with bytes
	pub fn create_rgba_texture(name: String, width: u16, height: u16, pixels: &Vec<u8>) -> Self {
		let mut texture = Self {
			width,
			height,
			id: 0,
			internal_format: gl::RGBA,
			name,
			format: gl::RGBA,
			data_type: gl::UNSIGNED_BYTE,
		};
		// Create the OpenGL texture and set it's data to null since it's empty
		unsafe {
			gl::GenTextures(1, &mut texture.id as *mut u32);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				texture.internal_format as i32,
				width as i32,
				height as i32,
				0,
				texture.format,
				texture.data_type,
				pixels.as_ptr() as *const c_void,
			);
			// Mag and min filters
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		}

		return texture;
	}
	// Update the size of this specific texture
	pub fn update_size(&mut self, xsize: u32, ysize: u32) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.id);
			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				self.internal_format as i32,
				xsize as i32,
				ysize as i32,
				0,
				self.format,
				self.data_type,
				null(),
			);
		}
	}
}
