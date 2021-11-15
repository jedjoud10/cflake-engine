use crate::RenderingError;
use assets::*;
use bitflags::bitflags;
use gl;
use image::{DynamicImage, EncodableLayout, GenericImageView};

use std::{ffi::c_void, ptr::null, rc::Rc};

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
    Texture1D(u16),
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
#[derive(Debug, Clone)]
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
            internal_format: gl::RGBA8,
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
    // Load a texture from scratch
    fn asset_load(data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let (bytes, width, height) = Self::read_bytes(data);
        // Return a texture with the default parameters
        let texture = Texture::new()
            .set_dimensions(TextureType::Texture2D(width, height))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_name(&data.name)
            .generate_texture(bytes)
            .unwrap();
        return Some(texture);
    }
    // Load a texture that already has it's parameters set
    fn asset_load_t(self, data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        let (bytes, width, height) = Self::read_bytes(data);
        let texture = self.set_name(&data.name).set_dimensions(TextureType::Texture2D(width, height));
        // Return a texture with the default parameters
        let texture = texture.generate_texture(bytes).unwrap();
        return Some(texture);
    }
}

impl Object for Texture {}

impl Texture {
    // New
    pub fn new() -> Self {
        Self::default()
    }
    // Read the texture bytes from a specific AssetMetadata
    fn read_bytes(metadata: &AssetMetadata) -> (Vec<u8>, u16, u16) {
        // Load this texture from the bytes
        let png_bytes = metadata.bytes.as_bytes();
        let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
        // Flip
        let image = image.flipv();
        return (image.to_bytes(), image.width() as u16, image.height() as u16);
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
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, self.id);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, self.internal_format as i32, width as i32, 0, self.format, self.data_type, null());
                }
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
    // Guess how many mipmap levels a texture with a specific maximum coordinate can have
    pub fn guess_mipmap_levels(i: usize) -> usize {
        let mut x: f32 = i as f32;
        let mut num: usize = 0;
        while x > 1.0 {
            // Repeatedly divide by 2
            x = x / 2.0;
            num += 1;
        }
        num
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
    pub fn apply_texture_load_options(self, opt: Option<TextureLoadOptions>) -> Texture {
        let opt = opt.unwrap_or_default();
        let texture = self.set_filter(opt.filter);
        let texture = texture.set_wrapping_mode(opt.wrapping);
        return texture;
    }
    // Create a texture array from multiple texture paths (They must have the same dimensions!)
    pub fn create_texturearray(load_options: Option<TextureLoadOptions>, texture_paths: Vec<&str>, asset_manager: &mut AssetManager, width: u16, height: u16) -> Rc<Texture> {
        // Load the textures
        let mut bytes: Vec<u8> = Vec::new();
        let name = &format!("{}-{}", "2dtexturearray", texture_paths.join("--"));
        let length = texture_paths.len();
        for x in texture_paths {
            // Load this texture from the bytes
            let metadata = asset_manager.asset_cacher.load_md(x).unwrap();
            let png_bytes = metadata.bytes.as_bytes();
            let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
            // Resize the image so it fits the dimension criteria
            let image = image.resize_exact(width as u32, height as u32, image::imageops::FilterType::Gaussian);
            // Flip
            let image = image.flipv();
            let bytesa = image.to_bytes();
            bytes.extend(bytesa);
        }
        // Create the array texture from THOSE NUTS AAAAA
        let main_texture: Texture = Texture::new()
            .enable_mipmaps()
            .set_dimensions(TextureType::TextureArray(width, height, length as u16))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_name(name)
            .apply_texture_load_options(load_options)
            .generate_texture(bytes)
            .unwrap();
        main_texture.object_cache_load(name, &mut asset_manager.object_cacher)
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    pub fn generate_texture(mut self, bytes: Vec<u8>) -> Result<Self, RenderingError> {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
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
                    TextureType::Texture1D(_) => {
                        gl::TexImage1D(tex_type, 0, self.internal_format as i32, self.get_width() as i32, 0, self.format, self.data_type, pointer);
                    }
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
                    TextureType::TextureArray(x, y, l) => {
                        gl::TexStorage3D(
                            tex_type,
                            Self::guess_mipmap_levels(x.max(y) as usize) as i32,
                            self.internal_format,
                            x as i32,
                            y as i32,
                            l as i32,
                        );
                        // We might want to do mipmap
                        for i in 0..l {
                            let localized_bytes = bytes[(i as usize * y as usize * 4 * x as usize)..bytes.len()].as_ptr() as *const c_void;
                            gl::TexSubImage3D(
                                gl::TEXTURE_2D_ARRAY,
                                0,
                                0,
                                0,
                                i as i32,
                                self.get_width() as i32,
                                self.get_height() as i32,
                                1,
                                self.format,
                                self.data_type,
                                localized_bytes,
                            );
                        }
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
        println!("Succsesfully generated texture {}", self.name);
        Ok(self)
    }
    // Update a valid texture's data
    pub fn update_data(&mut self, bytes: Vec<u8>) {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        unsafe {
            let tex_type = match self.ttype {
                TextureType::Texture1D(_) => gl::TEXTURE_1D,
                TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
                TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
                TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
            };
            gl::BindTexture(tex_type, self.id);
            match self.ttype {
                TextureType::Texture1D(_) => gl::TexImage1D(tex_type, 0, self.internal_format as i32, self.get_width() as i32, 0, self.format, self.data_type, pointer),
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
                TextureType::TextureArray(x, y, l) => {
                    gl::TexStorage3D(tex_type, 10, self.internal_format, x as i32, y as i32, l as i32);
                    // We might want to do mipmap
                    for i in 0..l {
                        let localized_bytes = bytes[(i as usize * y as usize * 4 * x as usize)..bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(
                            gl::TEXTURE_2D_ARRAY,
                            0,
                            0,
                            0,
                            i as i32,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            1,
                            self.format,
                            self.data_type,
                            localized_bytes,
                        );
                    }
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
    }
    // Get the image from this texture and fill an array of vec2s, vec3s or vec4s with it
    pub fn fill_array_veclib<V, U>(&self) -> Vec<V>
    where
        V: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Get the length of the vector
        let length: usize = match self.ttype {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<V> = vec![V::default(); length];

        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, self.id);
            gl::GetTexImage(tex_type, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
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
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<U> = vec![U::default(); length];

        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, self.id);
            gl::GetTexImage(tex_type, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
        }
        return pixels;
    }
    // Get the width of this texture
    pub fn get_width(&self) -> u16 {
        match self.ttype {
            TextureType::Texture1D(x) => x,
            TextureType::Texture2D(x, _) => x,
            TextureType::Texture3D(x, _, _) => x,
            TextureType::TextureArray(x, _, _) => x,
        }
    }
    // Get the height of this texture
    pub fn get_height(&self) -> u16 {
        match self.ttype {
            TextureType::Texture1D(y) => panic!(),
            TextureType::Texture2D(_, y) => y,
            TextureType::Texture3D(_, y, _) => y,
            TextureType::TextureArray(_, y, _) => y,
        }
    }
    // Get the depth of this texture, if it is a 3D texture
    pub fn get_depth(&self) -> u16 {
        match self.ttype {
            TextureType::Texture1D(_) => panic!(),
            TextureType::Texture2D(_, _) => panic!(),
            TextureType::Texture3D(_, _, z) => z,
            TextureType::TextureArray(_, _, z) => z,
        }
    }
    // Load a texture, and cache it if needed
    pub fn cache_load(self, local_path: &str, asset_manager: &mut AssetManager) -> Rc<Self> {
        // Early
        if asset_manager.object_cacher.cached(local_path) {
            return self.object_load_ot(local_path, &asset_manager.object_cacher).unwrap();
        }
        // Load the asset first
        let texture = self.asset_load_easy_t(local_path, &mut asset_manager.asset_cacher).unwrap();
        // Then the object (cache it if neccessarry)
        let output = texture.object_cache_load(local_path, &mut asset_manager.object_cacher);
        output
    }
}
