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
pub enum TextureWrapping {
    // TODO: Border colors directly here
    ClampToEdge,
    ClampToBorder,
    Repeat,
    MirroredRepeat,
}

// Texture dimensions
#[derive(EnumAsInner, Debug, Clone, Copy)]
pub enum TextureDimensions {
    Texture1D(u16),
    Texture2D(veclib::Vector2<u16>),
    Texture3D(veclib::Vector3<u16>),
    Texture2DArray(veclib::Vector3<u16>),
}

// How we can access the texture
// We will make an Upload and Download PBO for each case
bitflags! {
    pub struct TextureAccessType: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}
