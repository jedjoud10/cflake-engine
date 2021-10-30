use bitflags::bitflags;
use gl;
use image::EncodableLayout;
use others::CacheManager;
use assets::*;

use std::{ffi::c_void, ptr::null};

bitflags! {
    pub struct TextureFlags: u8 {
        const MUTABLE = 0b00000001;
        const MIPMAPS = 0b00000010;
    }
}

// How we load texture
#[derive(Default, Clone, Copy)]
pub struct TextureLoadOptions {
    pub filter: TextureFilter,
    pub wrapping: TextureWrapping,
}
/*
impl TextureLoadOptions {
    pub fn get_hashed_state(&self) -> u64 {

    }
}
*/

// Texture filters
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

impl Default for TextureFilter {
    fn default() -> Self {
        Self::Linear
    }
}

// Texture wrapping filters
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapping {
    ClampToEdge,
    ClampToBorder,
    Repeat,
    MirroredRepeat,
}

impl Default for TextureWrapping {
    fn default() -> Self {
        Self::Repeat
    }
}

// Texture type
#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    Texture2D(u16, u16),
    Texture3D(u16, u16, u16),
    TextureArray(u16, u16, u16),
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
    pub ttype: TextureType,
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
            filter: TextureFilter::default(),
            wrap_mode: TextureWrapping::default(),
            ttype: TextureType::Texture2D(0, 0),
        }
    }
}

// Loadable asset
impl Asset for Texture {
    fn load(data: AssetMetadata) -> Self where Self: Sized {
        todo!()
    }
    // Load a texture from the bundled metadata
    /*
    fn from_resource(self, resource: &Resource) -> Option<Self> {
        match resource {
            Resource::Texture(texture, texture_name) => {
                // Load either a 2D texture or a custom 3D texture
                match self.ttype {
                    TextureType::Texture2D(_, _) => {
                        let width = texture.width;
                        let height = texture.height;

                        // Turn the compressed png bytes into their raw form
                        let mut image = image::io::Reader::new(std::io::Cursor::new(&texture.compressed_bytes));
                        image.set_format(image::ImageFormat::Png);
                        let decoded = image.with_guessed_format().unwrap().decode().unwrap();
                        // Well it seems like the images are flipped vertically so I have to manually flip them
                        let decoded = decoded.flipv();
                        // Read the image as a 32 bit image
                        let rgba8_image = decoded.to_rgba8();

                        // Set the proper dimensions and generate the texture from the resource's bytes
                        let mut texture = self.set_dimensions(TextureType::Texture2D(width, height));
                        // Set the texture name since the texture has an empty name
                        texture.name = texture_name.clone();
                        let new_texture = texture.generate_texture(rgba8_image.as_bytes().to_vec()).unwrap();
                        texture = new_texture;
                        Some(texture)
                    }
                    TextureType::Texture3D(_, _, _) => todo!(),
                    TextureType::TextureArray(_, _, _) => todo!(),
                }
            }
            _ => None,
        }
    }
    */
}

// Loading / caching stuff
impl Texture {
    // New
    pub fn new() -> Self {
        Self::default()
    }
    // Cache the current texture and return it's reference
    pub fn cache_texture<'a>(self, texture_cacher: &'a mut CacheManager<Texture>) -> Option<(&'a mut Self, usize)> {
        let texture_name = self.name.clone();
        // If the name is empty, cache it as an unnamed object
        if texture_name.trim().is_empty() {
            // Unnamed object
            let texture_id = texture_cacher.cache_unnamed_object(self);
            Some((texture_cacher.id_get_object_mut(texture_id).unwrap(), texture_id))
        } else {
            let texture_id = texture_cacher.cache_object(self, texture_name.as_str());
            Some((texture_cacher.id_get_object_mut(texture_id).unwrap(), texture_id))
        }
    }
    // Load a texture from a file and auto caches it. Returns the cached texture and the cached ID
    pub fn load_texture<'a>(
        self,
        local_path: &str,
        resource_manager: &AssetManager,
        texture_cacher: &'a mut CacheManager<Texture>,
    ) -> Result<(&'a Self, usize), AssetLoadError> {
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
            let texture = self.from_resource(resource).ok_or(Asset::new_str("Could not load texture!"))?;
            let (texture, texture_id) = texture.cache_texture(texture_cacher).unwrap();
            Ok((texture, texture_id))
        }
    }
}

impl Texture {
    // Set name
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    // The internal format and data type of the soon to be generated texture
    pub fn set_idf(mut self, internal_format: u32, format: u32, data_type: u32) -> Self {
        self.internal_format = internal_format;
        self.format = format;
        self.data_type = data_type;
        self
    }
    // Set the height and width of the soon to be generated texture
    pub fn set_dimensions(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Set the texture type
    pub fn set_type(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Update the size of the current texture
    pub fn update_size(&mut self, ttype: TextureType) {
        // Check if the current dimension type matches up with the new one
        if false { /* Oopsie woopsie, we did a little fucky wuckie, a little fucko boingo. The code monkey (Me) is working VEWWY hard to fix this >.<!! */ }
        self.ttype = ttype;
        // This is a normal texture getting resized
        unsafe {
            match self.ttype {
                TextureType::Texture2D(width, height) => {
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
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::TexImage3D(
                        gl::TEXTURE_3D,
                        0,
                        self.internal_format as i32,
                        width as i32,
                        height as i32,
                        depth as i32,
                        0,
                        self.format,
                        self.data_type,
                        null(),
                    );
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
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
    // Disable mipmaps
    pub fn disable_mipmaps(mut self) -> Self {
        self.flags &= !TextureFlags::MIPMAPS;
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
    // Apply the texture load options on a texture
    pub fn apply_texture_load_options(mut self, opt: Option<TextureLoadOptions>) -> Texture {
        let opt = opt.unwrap_or_default();
        let texture = self.set_filter(opt.filter);
        let texture = texture.set_wrapping_mode(opt.wrapping);
        return texture;
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Result<Self, errors::RenderingError> {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match self.ttype {
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        if true {
            // It's a normal mutable texture
            unsafe {
                gl::GenTextures(1, &mut self.id as *mut u32);
                gl::BindTexture(tex_type, self.id);
                match self.ttype {
                    // This is a 2D texture
                    TextureType::Texture2D(_, _) => {
                        gl::TexImage2D(
                            tex_type,
                            0,
                            self.internal_format as i32,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            0,
                            self.format,
                            self.data_type,
                            pointer,
                        );
                    }
                    // This is a 3D texture
                    TextureType::Texture3D(_, _, _) => {
                        gl::TexImage3D(
                            tex_type,
                            0,
                            self.internal_format as i32,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            self.get_depth() as i32,
                            0,
                            self.format,
                            self.data_type,
                            pointer,
                        );
                    }
                    // This is a texture array
                    TextureType::TextureArray(_, _, _) => {
                        gl::TexStorage3D(
                            tex_type,
                            1,
                            self.internal_format,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            self.get_depth() as i32,
                        );
                        gl::TexSubImage3D(
                            tex_type,
                            0,
                            0,
                            0,
                            0,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            self.get_depth() as i32,
                            self.format,
                            self.data_type,
                            pointer,
                        );
                    }
                }
                errors::ErrorCatcher::catch_opengl_errors().ok_or(errors::RenderingError::new_str("Failed texture creation!"))?;
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
                errors::ErrorCatcher::catch_opengl_errors().ok_or(errors::RenderingError::new_str("Failed to set texture filter!"))?;
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
                    errors::ErrorCatcher::catch_opengl_errors().ok_or(errors::RenderingError::new_str("Failed to set texture mipmap filter!"))?;
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
            errors::ErrorCatcher::catch_opengl_errors().ok_or(errors::RenderingError::new_str("Failed to set texture wrapping mode!"))?;
        }
        Ok(self)
    }
    // Get the image from this texture and fill an array of vec2s, vec3s or vec4s with it
    pub fn fill_array_veclib<V, U>(&self) -> Vec<V>
    where
        V: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Get the length of the vector
        let length: usize = match self.ttype {
            TextureType::Texture2D(x, y) => (x * y) as usize,
            TextureType::Texture3D(x, y, z) => (x * y * z) as usize,
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<V> = vec![V::default(); length];

        // Actually read the pixels
        unsafe {
            match self.ttype {
                TextureType::Texture2D(_, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_2D, self.id);
                    gl::GetTexImage(gl::TEXTURE_2D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureType::Texture3D(_, _, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::GetTexImage(gl::TEXTURE_3D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureType::TextureArray(_, _, _) => todo!(),
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
        let length: usize = match self.ttype {
            TextureType::Texture2D(x, y) => (x * y) as usize,
            TextureType::Texture3D(x, y, z) => (x * y * z) as usize,
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<U> = vec![U::default(); length];

        // Actually read the pixels
        unsafe {
            match self.ttype {
                TextureType::Texture2D(_, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_2D, self.id);
                    gl::GetTexImage(gl::TEXTURE_2D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureType::Texture3D(_, _, _) => {
                    // Bind the buffer before reading
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::GetTexImage(gl::TEXTURE_3D, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
        return pixels;
    }
    // Get the width of this texture
    pub fn get_width(&self) -> u16 {
        match self.ttype {
            TextureType::Texture2D(x, _) => x,
            TextureType::Texture3D(x, _, _) => x,
            TextureType::TextureArray(x, _, _) => x,
        }
    }
    // Get the height of this texture
    pub fn get_height(&self) -> u16 {
        match self.ttype {
            TextureType::Texture2D(_, y) => y,
            TextureType::Texture3D(_, y, _) => y,
            TextureType::TextureArray(_, y, _) => y,
        }
    }
    // Get the depth of this texture, if it is a 3D texture
    pub fn get_depth(&self) -> u16 {
        match self.ttype {
            TextureType::Texture2D(_, _) => todo!(),
            TextureType::Texture3D(_, _, z) => z,
            TextureType::TextureArray(_, _, z) => z,
        }
    }
}
