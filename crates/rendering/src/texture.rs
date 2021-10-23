use bitflags::bitflags;
use gl;

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

// Texture dimension type
#[derive(Debug, Clone)]
pub enum TextureDimensionType {
    D2D(u16, u16),
    D3D(u16, u16, u16),
}

// Custom internal format
// Custom data type
// Custom format

// Access type when binding the texture
#[derive(Clone, Copy)]
pub enum TextureShaderAccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

// A texture, could be 2D or 3D
#[derive(Debug)]
pub struct Texture {
    pub id: u32,
    pub name: String,
    pub internal_format: u32,
    pub format: u32,
    pub data_type: u32,
    pub flags: TextureFlags,
    pub filter: TextureFilter,
    pub wrap_mode: TextureWrapping,
    pub dimension_type: TextureDimensionType,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            internal_format: gl::RGBA,
            format: gl::RGBA,
            data_type: gl::UNSIGNED_BYTE,
            flags: TextureFlags::empty(),
            filter: TextureFilter::Linear,
            dimension_type: TextureDimensionType::D2D(0, 0),
            wrap_mode: TextureWrapping::Repeat,
        }
    }
}

impl Texture {
    // The internal format and data type of the soon to be generated texture
    pub fn set_idf(mut self, internal_format: u32, format: u32, data_type: u32) -> Self {
        self.internal_format = internal_format;
        self.format = format;
        self.data_type = data_type;
        self
    }
    // Set if we should use the new opengl api (Gl tex storage that allows for immutable texture) or the old one
    pub fn set_mutable(mut self, mutable: bool) -> Self {
        /*
        todo!();
        match mutable {
            true => self.flags |= TextureFlags::MUTABLE,
            false => self.flags &= !TextureFlags::MUTABLE,
        }
        */
        self
    }
    // Set the generation of mipmaps
    pub fn enable_mipmaps(mut self) -> Self {
        self.flags |= TextureFlags::MIPMAPS;
        self
    }
    // Set the mag and min filters
    pub fn set_filter(mut self, filter: TextureFilter) -> Self {
        self.filter = filter;
        self
    }
    // Set the wrapping mode
    pub fn set_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        self.wrap_mode = wrapping_mode;
        self
    }
    // Set the flags
    pub fn set_flags(mut self, flags: TextureFlags) -> Self {
        self.flags = flags;
        self
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>, dimension_type: TextureDimensionType) -> Self {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match dimension_type {
            TextureDimensionType::D2D(_, _) => gl::TEXTURE_2D,
            TextureDimensionType::D3D(_, _, _) => gl::TEXTURE_3D,
        };

        if true {
            // It's a normal mutable texture
            unsafe {
                gl::GenTextures(1, &mut self.id as *mut u32);
                gl::BindTexture(tex_type, self.id);
                // Use TexImage3D if it's a 3D texture, otherwise use TexImage2D
                match dimension_type {
                    // This is a 2D texture
                    TextureDimensionType::D2D(width, height) => {
                        gl::TexImage2D(
                            tex_type,
                            0,
                            self.internal_format as i32,
                            width as i32,
                            height as i32,
                            0,
                            self.format,
                            self.data_type,
                            pointer,
                        );
                    }
                    // This is a 3D texture
                    TextureDimensionType::D3D(width, height, depth) => {
                        gl::TexImage3D(
                            tex_type,
                            0,
                            self.internal_format as i32,
                            width as i32,
                            height as i32,
                            depth as i32,
                            0,
                            self.format,
                            self.data_type,
                            pointer,
                        );
                    }
                }

                // Set the texture parameters for a normal texture
                match self.filter {
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
                    match self.filter {
                        TextureFilter::Linear => {
                            // 'Linear' filter
                            gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                            gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                        }
                        TextureFilter::Nearest => {
                            // 'Nearest' filter
                            gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                            gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                        }
                    }
                }
            }
        } else {
            // Nobody loves you, OpenGL storage textures
            if self.flags.contains(TextureFlags::MIPMAPS) {
                // Create the mipmaps
                /*
                unsafe {
                    //gl::GenerateMipmap(tex_type);
                }
                */
            }
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode: i32;
        match self.wrap_mode {
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
    // Get the image from this texture and fill an array of vec2s, vec3s or vec4s with it
    pub fn fill_array_veclib<V, U>(&self) -> Vec<V>
    where
        V: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Get the length of the vector
        let length: usize = match self.dimension_type {
            TextureDimensionType::D2D(x, y) => (x * y) as usize,
            TextureDimensionType::D3D(x, y, z) => (x * y * z) as usize,
        };
        // Create the vector
        let mut pixels: Vec<V> = vec![V::default(); length];

        // Actually read the pixels
        unsafe {
            match self.dimension_type {
                TextureDimensionType::D2D(_, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_2D, self.id);
                    gl::GetTexImage(gl::TEXTURE_2D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureDimensionType::D3D(_, _, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::GetTexImage(gl::TEXTURE_3D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
            }
        }
        return pixels;
    }
    // Get the image from this texture and fill an array of single elements with it
    pub fn fill_array_elems<U>(&self) -> Vec<U>
    where
        U: Clone + Default,
    {
        // Get the length of the vector
        let length: usize = match self.dimension_type {
            TextureDimensionType::D2D(x, y) => (x * y) as usize,
            TextureDimensionType::D3D(x, y, z) => (x * y * z) as usize,
        };
        // Create the vector
        let mut pixels: Vec<U> = vec![U::default(); length];

        // Actually read the pixels
        unsafe {
            match self.dimension_type {
                TextureDimensionType::D2D(_, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_2D, self.id);
                    gl::GetTexImage(gl::TEXTURE_2D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureDimensionType::D3D(_, _, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::GetTexImage(gl::TEXTURE_3D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
            }
        }
        return pixels;
    }
}
