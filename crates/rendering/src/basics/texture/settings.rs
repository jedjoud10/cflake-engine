use bitflags::bitflags;
use enum_as_inner::EnumAsInner;

// Texture filters
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

// Texture wrapping filters
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapMode {
    ClampToEdge(Option<veclib::Vector4<f32>>),
    ClampToBorder(Option<veclib::Vector4<f32>>),
    Repeat,
    MirroredRepeat,
}

// Texture dimensions
#[derive(EnumAsInner, Debug, Clone, Copy)]
pub enum TextureDimensions {
    // TODO: I hate this
    Texture1d(u16),
    Texture2d(veclib::Vector2<u16>),
    Texture3d(veclib::Vector3<u16>),
    Texture2dArray(veclib::Vector3<u16>),
}

// How we can access the texture
// We will make an Upload and Download PBO for each case
bitflags! {
    pub struct TextureAccessType: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}
