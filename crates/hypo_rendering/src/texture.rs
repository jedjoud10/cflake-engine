use bitflags::bitflags;
use gl;
use hypo_others::CacheManager;
use hypo_resources::{LoadableResource, Resource, ResourceManager};
use image::EncodableLayout;

use std::{ffi::c_void, ptr::null};

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

// A texture trait that will be implemented on the Texture2D and Texture3D structs
pub trait Texture {
    // Get the current internal format
    fn get_internal_format_mut(&mut self) -> &mut u32;
    // Get the current format
    fn get_format_mut(&mut self) -> &mut u32;
    // Get the current data type
    fn get_data_type_mut(&mut self) -> &mut u32;
    // Get the current texture flags
    fn get_texture_flags_mut(&mut self) -> &mut TextureFlags;
    // Get the current texture filter
    fn get_texture_filter_mut(&mut self) -> &mut TextureFilter;
    // Get the current texture wrap mode
    fn get_texture_wrap_mode_mut(&mut self) -> &mut TextureWrapping;
    // Get the texture dimension type, either 2D or 3D (OpenGl)
    fn get_texture_dimension_type(&self) -> i32;
    // The internal format and data type of the soon to be generated texture
    fn set_idf(mut self, internal_format: u32, format: u32, data_type: u32) -> Self {
        self.get_internal_format_mut() = internal_format;
        self.get_format_mut() = format;
        self.get_data_type_mut() = data_type;
        self
    }
    // Set if we should use the new opengl api (Gl tex storage that allows for immutable texture) or the old one
    fn set_mutable(mut self, mutable: bool) -> Self {
        let result = self.get_texture_flags_mut();
        match mutable {
            true => result |= TextureFlags::MUTABLE,
            false => result &= !TextureFlags::MUTABLE,
        }
        self
    }
    // Set the generation of mipmaps
    fn enable_mipmaps(mut self) -> Self {
        self.get_texture_flags_mut() |= TextureFlags::MIPMAPS;
        self
    }
    // Set the mag and min filters
    fn set_filter(mut self, filter: TextureFilter) -> Self {
        self.get_texture_filter_mut() = filter;
        self
    }
    // Set the wrapping mode
    fn set_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        self.get_texture_wrap_mode_mut() = wrapping_mode;
        self
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    fn generate_texture(mut self, bytes: Vec<u8>) -> Self {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }
        let tex_type = self.get_texture_dimension_type();

        if self.flags.contains(TextureFlags::MUTABLE) {
            // It's a normal mutable texture
            unsafe {
                gl::GenTextures(1, &mut self.id as *mut u32);
                gl::BindTexture(tex_type, self.id);
                gl::TexImage2D(
                    tex_type,
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
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    }
                }
            }

            // The texture is already bound to the TEXTURE_2D
            if self.flags.contains(TextureFlags::MIPMAPS) {
                // Create the mipmaps
                unsafe {
                    gl::GenerateMipmap(tex_type);
                    // Set the texture parameters for a mipmapped texture
                    match self.texture_filter {
                        TextureFilter::Linear => {
                            // 'Linear' filter
                            gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                            gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                        }
                        TextureFilter::Nearest => {
                            // 'Nearest' filter
                            gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                            gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                        }
                    }
                }
            }
        } else {
            // Nobody loves you, OpenGL storage textures
            if self.flags.contains(TextureFlags::MIPMAPS) {
                // Create the mipmaps
                unsafe {
                    //gl::GenerateMipmap(tex_type);
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
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode);
        }
        self
    }
}

