use crate::utils::DataType;
use super::TextureFormat;

// Texture layout
#[derive(Clone, Copy)]
pub struct TextureLayout {
    pub data_type: DataType,
    pub internal_format: TextureFormat,
    pub resizable: bool,
}