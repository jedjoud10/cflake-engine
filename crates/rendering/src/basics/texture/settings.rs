use enum_as_inner::EnumAsInner;
use bitflags::bitflags;

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

// Texture parameter bits
bitflags! {
    pub struct TextureBits: u8 {
        const MIPMAPS = 1;
        const SRGB = 1 << 1;
    }
}