use super::{TextureLayout};
use bitflags::bitflags;
use enum_as_inner::EnumAsInner;

// Texture parameter bits
bitflags! {
    pub struct TextureFlags: u8 {
        const MIPMAPS = 1;
        const SRGB = 1 << 1;
        const RESIZABLE = 1 << 2;
        const PERSISTENT = 1 << 3;
    }
}

// Texture bytes
#[derive(EnumAsInner)]
pub enum TextureBytes {
    Loaded(Vec<u8>),
    Unloaded,
}

// Texture filters
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

// Texture wrapping filters
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapMode {
    ClampToEdge(Option<vek::Vec4<f32>>),
    ClampToBorder(Option<vek::Vec4<f32>>),
    Repeat,
    MirroredRepeat,
}

// Texture parameters 
pub struct TextureParams {
    // Loaded texture bytes
    pub bytes: TextureBytes,
    // Texture layout
    pub layout: TextureLayout,
    // Texture mag and min filters, either Nearest or Linear
    pub filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    pub wrap: TextureWrapMode,
    // Bits
    pub flags: TextureFlags,
}

impl TextureParams {
    // Load from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes: TextureBytes::Loaded(bytes),
            ..Default::default()
        }
    }
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            bytes: TextureBytes::Unloaded,
            layout: TextureLayout::default(),
            filter: TextureFilter::Linear,
            wrap: TextureWrapMode::Repeat,
            flags: TextureFlags::empty(),
        }
    }
}