use super::{TextureWrapMode, TextureFilter, TextureLayout, TextureBits, get_ifd};

// Texture parameters 
pub struct TextureParams {
    // Texture layout
    pub layout: TextureLayout,
    // Texture mag and min filters, either Nearest or Linear
    pub filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    pub wrap: TextureWrapMode,
    // Bits
    pub bits: TextureBits,
}

impl TextureParams {
    fn default() -> Self {
        Self {
            layout: TextureLayout::default(),
            filter: TextureFilter::Linear,
            wrap: TextureWrapMode::Repeat,
            bits: TextureBits::empty(),
        }
    }
}