use super::{Texture, TextureMode};

// Texel filters that are applied to the texture's mininifcation and magnification parameters
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Filter {
    // Filtering for any texture
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,

    // Filtering for textures that use mipmaps
    TryMipMapNearest = gl::NEAREST_MIPMAP_NEAREST,
    TryMipMapLinear = gl::LINEAR_MIPMAP_LINEAR,
}

// Wrapping mode utilised by TEXTURE_WRAP_R and TEXTURE_WRAP_T
#[derive(Clone, Copy)]
pub enum Wrap {
    // Oop sorry no more custom discriminent :(
    ClampToEdge,
    ClampToBorder(vek::Rgba<f32>),
    Repeat,
    MirroredRepeat,
}

// Some special sampling parameters for textures
#[derive(Clone, Copy)]
pub struct Sampling {
    pub(super) filter: Filter,
    pub(super) wrap: Wrap,
}

impl Sampling {
    // Create some new smapling parameters
    pub fn new(filter: Filter, wrap: Wrap) -> Self {
        Self { filter, wrap }
    }
}