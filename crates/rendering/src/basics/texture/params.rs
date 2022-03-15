use std::ptr::null;

use super::TextureLayout;
use bitflags::bitflags;
use enum_as_inner::EnumAsInner;
use gl::types::GLuint;

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
#[derive(EnumAsInner, Debug)]
pub enum TextureBytes {
    Valid(Vec<u8>),
    Invalid,
}

impl TextureBytes {
    // Clear the bytes and deallocate them
    pub fn clear(&mut self) {
        if let Self::Valid(bytes) = self {
            *bytes = Vec::new();
        }
    }
    // Pointer
    pub fn get_ptr(&self) -> *const u8 {
        if let Some(bytes) = self.as_valid() {
            if bytes.is_empty() {
                null()
            } else {
                bytes.as_ptr()
            }
        } else {
            null()
        }
    }
}

impl Default for TextureBytes {
    fn default() -> Self {
        Self::Invalid
    }
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
#[derive(Clone)]
pub struct TextureParams {
    // Texture layout
    pub layout: TextureLayout,
    // Texture mag and min filters, either Nearest or Linear
    pub filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    pub wrap: TextureWrapMode,
    // Custom
    pub custom: Vec<(GLuint, GLuint)>,
    // Bits
    pub flags: TextureFlags,
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            filter: TextureFilter::Linear,
            wrap: TextureWrapMode::Repeat,
            custom: Default::default(),
            flags: TextureFlags::MIPMAPS | TextureFlags::SRGB,
        }
    }
}
