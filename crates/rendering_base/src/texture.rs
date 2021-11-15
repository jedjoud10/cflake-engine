use crate::{error::RenderingError, main_types::DataType};

use bitflags::bitflags;
// The texture format
#[derive(Clone, Copy)]
pub enum TextureFormat {
    // Red
    R8,
    R16,
    R32,
    // FP
    R16F,
    R32f,
    // Red Green
    RG8,
    RG16,
    RG32,
    // FP
    RG16F,
    RG32F,
    // Red Green Blue
    RGB8,
    RGB16,
    RGB32,
    // FP
    RGB16F,
    RGB32F,
    // Red Green Blue Alpha
    RGBA8,
    RGBA16,
    RGBA32,
    // FP
    RGBA16F,
    RGBA32F
}


bitflags! {
    pub struct TextureFlags: u8 {
        const MUTABLE = 0b00000001;
        const MIPMAPS = 0b00000010;
    }
}

// How we load texture
#[derive( Clone, Copy)]
pub struct TextureLoadOptions {
    pub filter: TextureFilter,
    pub wrapping: TextureWrapping,
}

impl Default for TextureLoadOptions {
    fn default() -> Self {
        Self { 
            filter: TextureFilter::Linear,
            wrapping: TextureWrapping::Repeat
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

// Access type when binding an image to a compute shader per say
#[derive(Clone, Copy)]
pub enum TextureShaderAccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

// A texture, could be 2D or 3D
#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub name: String,
    pub _format: TextureFormat, 
    pub _type: DataType,
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
            _format: TextureFormat::RGBA8,
            _type: DataType::UByte,
            flags: TextureFlags::empty(),
            filter: TextureFilter::Linear,
            wrap_mode: TextureWrapping::Repeat,
            ttype: TextureType::Texture2D(0, 0),
        }
    }
}

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
}