use super::TextureFormat;
use crate::utils::DataType;

// Texture layout that depicts how the texture will be read/written
#[derive(Clone, Copy)]
pub struct TextureLayout {
    pub(crate) data: DataType,
    pub(crate) internal_format: TextureFormat,
}

impl TextureLayout {
    // Texture layout for textures that get loaded at runtime
    pub const LOADED: Self = Self::new(DataType::U8, TextureFormat::RGBA8R);

    pub const fn new(data: DataType, _format: TextureFormat) -> TextureLayout {
        Self { data, internal_format: _format }
    }
}