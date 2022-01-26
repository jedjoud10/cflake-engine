
use bitflags::bitflags;

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
    Texture2DArray(u16, u16, u16),
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
            TextureType::Texture2DArray(x, _, _) => *x,
        }
    }
    // Get the height of this texture
    pub fn get_height(&self) -> u16 {
        match self {
            TextureType::Texture1D(_y) => panic!(),
            TextureType::Texture2D(_, y) => *y,
            TextureType::Texture3D(_, y, _) => *y,
            TextureType::Texture2DArray(_, y, _) => *y,
        }
    }
    // Get the depth of this texture, if it is a 3D texture
    pub fn get_depth(&self) -> u16 {
        match self {
            TextureType::Texture1D(_) => panic!(),
            TextureType::Texture2D(_, _) => panic!(),
            TextureType::Texture3D(_, _, z) => *z,
            TextureType::Texture2DArray(_, _, z) => *z,
        }
    }
}
// How we can access the texture
// We will make an Upload and Download PBO for each case
bitflags! {
    pub struct TextureAccessType: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}