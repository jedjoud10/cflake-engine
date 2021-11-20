use crate::{utils::*, GPUObject};
use assets::*;
use bitflags::bitflags;
use gl;
use image::{EncodableLayout, GenericImageView};
use std::{ffi::c_void, ptr::null};
bitflags! {
    pub struct TextureFlags: u8 {
        const MUTABLE = 0b00000001;
        const MIPMAPS = 0b00000010;
    }
}

// How we load texture
#[derive(Clone, Copy)]
pub struct TextureLoadOptions {
    pub filter: TextureFilter,
    pub wrapping: TextureWrapping,
}

impl Default for TextureLoadOptions {
    fn default() -> Self {
        Self {
            filter: TextureFilter::Linear,
            wrapping: TextureWrapping::Repeat,
        }
    }
}

// Texture filters
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

// Texture wrapping filters
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapping {
    ClampToEdge,
    ClampToBorder,
    Repeat,
    MirroredRepeat,
}

// Texture type
#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    Texture1D(u16),
    Texture2D(u16, u16),
    Texture3D(u16, u16, u16),
    TextureArray(u16, u16, u16),
}

impl Default for TextureType {
    fn default() -> Self {
        Self::Texture2D(0, 0)
    }
}

impl TextureType {
    // Get the width of this texture
    pub fn get_width(&self) -> u16 {
        match self {
            TextureType::Texture1D(x) => *x,
            TextureType::Texture2D(x, _) => *x,
            TextureType::Texture3D(x, _, _) => *x,
            TextureType::TextureArray(x, _, _) => *x,
        }
    }
    // Get the height of this texture
    pub fn get_height(&self) -> u16 {
        match self {
            TextureType::Texture1D(y) => panic!(),
            TextureType::Texture2D(_, y) => *y,
            TextureType::Texture3D(_, y, _) => *y,
            TextureType::TextureArray(_, y, _) => *y,
        }
    }
    // Get the depth of this texture, if it is a 3D texture
    pub fn get_depth(&self) -> u16 {
        match self {
            TextureType::Texture1D(_) => panic!(),
            TextureType::Texture2D(_, _) => panic!(),
            TextureType::Texture3D(_, _, z) => *z,
            TextureType::TextureArray(_, _, z) => *z,
        }
    }
}

// Access type when binding an image to a compute shader per say
#[derive(Clone, Copy)]
pub enum TextureShaderAccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

// A texture
#[derive(Clone)]
pub struct Texture {
    // The internal GPU Object for this texture
    pub name: String,
    pub bytes: Vec<u8>,
    pub _format: TextureFormat, // The internal format of the texture
    pub _type: DataType,        // The data type that this texture uses for storage
    pub flags: TextureFlags,
    pub filter: TextureFilter, // Texture mag and min filters, either Nearest or Linear
    pub wrap_mode: TextureWrapping,
    pub ttype: TextureType, // The dimensions of the texture and it's texture type
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            name: String::new(),
            bytes: Vec::new(),
            _format: TextureFormat::RGBA8R,
            _type: DataType::UByte,
            flags: TextureFlags::empty(),
            filter: TextureFilter::Linear,
            wrap_mode: TextureWrapping::Repeat,
            ttype: TextureType::Texture2D(0, 0),
        }
    }
}

// Load

// Some texture-only things, not related to OpenGL
impl Texture {
    // Set name
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    // The internal format and data type of the soon to be generated texture
    pub fn set_format(mut self, _format: TextureFormat) -> Self {
        self._format = _format;
        self
    }
    // Set the data type for this texture
    pub fn set_data_type(mut self, _type: DataType) -> Self {
        self._type = _type;
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
    // Set if we should use the new opengl api (Gl tex storage that allows for immutable texture) or the old one
    pub fn set_mutable(self, _mutable: bool) -> Self {
        /*
        todo!();
        match mutable {
            true => self.flags |= TextureFlags::MUTABLE,
            false => self.flags &= !TextureFlags::MUTABLE,
        }
        */
        self
    }
    // Apply the texture load options on a texture
    pub fn apply_texture_load_options(self, opt: Option<TextureLoadOptions>) -> Texture {
        let opt = opt.unwrap_or_default();
        let texture = self.set_filter(opt.filter);

        texture.set_wrapping_mode(opt.wrapping)
    }
    // Cr
    // Guess how many mipmap levels a texture with a specific maximum coordinate can have
    pub fn guess_mipmap_levels(i: usize) -> usize {
        let mut x: f32 = i as f32;
        let mut num: usize = 0;
        while x > 1.0 {
            // Repeatedly divide by 2
            x /= 2.0;
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
            TextureType::Texture1D(_y) => panic!(),
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
}

impl Texture {
    // Read bytes
    pub fn read_bytes(metadata: &AssetMetadata) -> (Vec<u8>, u16, u16) {
        // Load this texture from the bytes
        let png_bytes = metadata.bytes.as_bytes();
        let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
        // Flip
        let image = image.flipv();
        (image.to_bytes(), image.width() as u16, image.height() as u16)
    }
    // Create a texture array from multiple texture paths (They must have the same dimensions!)
    pub fn create_texturearray(
        load_options: Option<TextureLoadOptions>,
        texture_paths: Vec<&str>,
        asset_manager: &mut AssetManager,
        width: u16,
        height: u16,
    ) -> (Vec<Vec<u8>>, TextureType) {
        // Load the textures
        let mut bytes: Vec<Vec<u8>> = Vec::new();
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
            bytes.push(bytesa);
        }
        (bytes, TextureType::TextureArray(width, height, texture_paths.len() as u16))
    }
}
