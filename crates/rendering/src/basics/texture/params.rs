use std::ptr::null;

use crate::utils::DataType;

use super::{TextureFormat, TextureLayout};
use bitflags::bitflags;
use enum_as_inner::EnumAsInner;

// Texture parameter bits
bitflags! {
    pub struct TextureFlags: u8 {
        const MIPMAPS = 1;
        const SRGB = 1 << 1;
        const RESIZABLE = 1 << 2;
        const PERSISTENT = 1 << 3;
        const ANISOTROPIC = 1 << 4;
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
            bytes.clear();
            bytes.shrink_to_fit();
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
    ClampToEdge,
    ClampToBorder(Option<vek::Rgba<f32>>),
    Repeat,
    MirroredRepeat,
}

// Texture parameters
#[derive(Clone, Copy)]
pub struct TextureParams {
    pub layout: TextureLayout,
    pub filter: TextureFilter,
    pub wrap: TextureWrapMode,
    pub flags: TextureFlags,
}

impl TextureParams {
    // Parameters when loading an SRGB diffuse map
    pub const DIFFUSE_MAP_LOAD: Self = Self {
        layout: TextureLayout::LOADED,
        filter: TextureFilter::Linear,
        wrap: TextureWrapMode::Repeat,
        flags: TextureFlags::from_bits_truncate(TextureFlags::MIPMAPS.bits | TextureFlags::SRGB.bits | TextureFlags::ANISOTROPIC.bits),
    };
    // Parameters when loading a map that doesn't contain color data
    pub const NON_COLOR_MAP_LOAD: Self = Self {
        layout: TextureLayout::LOADED,
        filter: TextureFilter::Linear,
        wrap: TextureWrapMode::Repeat,
        flags: TextureFlags::from_bits_truncate(TextureFlags::MIPMAPS.bits | TextureFlags::ANISOTROPIC.bits),
    };
    // Parameters when loading an HDR texture
    pub const HDR_MAP_LOAD: Self = Self {
        layout: TextureLayout::HDR,
        filter: TextureFilter::Linear,
        wrap: TextureWrapMode::ClampToEdge,
        flags: TextureFlags::from_bits_truncate(TextureFlags::RESIZABLE.bits),
    };
}
