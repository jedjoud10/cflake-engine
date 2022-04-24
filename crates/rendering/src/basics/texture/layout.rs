use super::TextureFormat;
use crate::utils::DataType;

// Texture layout that depicts how the texture will be read/written
#[derive(Clone, Copy, PartialEq)]
pub struct TextureLayout {
    pub(crate) data: DataType,
    pub(crate) internal_format: TextureFormat,
}

impl TextureLayout {
    // Texture layout for textures that get loaded at runtime
    pub const LOADED: Self = Self::new(DataType::U8, TextureFormat::RGBA8R);

    // Texture layout for HDR textures
    pub const HDR: Self = Self::new(DataType::U8, TextureFormat::RGB32F);

    pub const fn new(data: DataType, _format: TextureFormat) -> TextureLayout {
        Self { data, internal_format: _format }
    }
}
