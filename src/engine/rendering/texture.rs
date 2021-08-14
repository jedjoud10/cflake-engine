use crate::engine::core::cacher::CacheManager;
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
        local_path: &str,
        resource_manager: &mut ResourceManager,
        texture_manager: &mut CacheManager<Texture>,
    ) -> Option<u16> {
		// Load the resource
		let resource = resource_manager.load_packed_resource(local_path)?;
		// Then load the texture from that resource
		let texture = Texture::from_resource(resource)?;
		let texture_id = texture_manager.cache_object(texture, local_path);
		return Some(texture_id);
    }
    // Convert the resource to a texture
    pub fn from_resource(resource: &Resource) -> Option<Self> {
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

                let mut new_texture = Self::create_rgba_texture(
                    texture_name.clone(),
                    width,
                    height,
                    &rgba8_image.as_bytes().to_owned(),
                );
                new_texture.name = texture_name.clone();
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
