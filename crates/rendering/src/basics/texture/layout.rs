use super::TextureFormat;
use crate::utils::DataType;

// Texture layout
#[derive(Clone, Copy)]
pub struct TextureLayout {
    pub data: DataType,
    pub internal_format: TextureFormat,
}

impl TextureLayout {
    pub fn new(data: DataType, _format: TextureFormat) -> TextureLayout {
        Self {
            data, internal_format: _format,
        }
    }
}

impl Default for TextureLayout {
    fn default() -> Self {
        Self {
            data: DataType::U8,
            internal_format: TextureFormat::RGBA8R,
        }
    }
}
