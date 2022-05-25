use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
    num::NonZeroU32,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    context::Context,
    object::{ToGlName, ToGlType},
};

use super::{Bindless, TexelLayout, Texture, TextureMode};

// Texel filters that are applied to the texture's mininifcation and magnification parameters
#[repr(u32)]
pub enum Filter {
    // Filtering for any texture
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,

    // Filtering for textures that use mipmaps
    TryMipMapNearest = gl::NEAREST_MIPMAP_NEAREST,
    TryMipMapLinear = gl::LINEAR_MIPMAP_LINEAR,
}

// Wrapping mode utilised by TEXTURE_WRAP_R and TEXTURE_WRAP_T
pub enum Wrap {
    // Oop sorry no more custom discriminent :(
    ClampToEdge,
    ClampToBorder(vek::Rgba<f32>),
    Repeat,
    MirroredRepeat,
}

// Some special sampling parameters for textures
pub struct Sampling {
    filter: Filter,
    wrap: Wrap,
}

impl Sampling {
    // Create some new smapling parameters
    pub fn new(filter: Filter, wrap: Wrap) -> Self {
        Self { filter, wrap }
    }
}

// Apply some sampling parameters to a specific texture
pub(super) unsafe fn apply(name: u32, target: u32, mode: TextureMode, sampling: Sampling) {
    // We do a bit of enum fetching (this is safe) (trust)
    let filter = std::mem::transmute::<Filter, u32>(sampling.filter);

    // Min and mag filters conversion cause OpenGL suxs
    let min = filter as i32;
    let mag = filter as i32;

    // Set the filters
    gl::TextureParameteri(name, gl::TEXTURE_MIN_FILTER, min);
    gl::TextureParameteri(name, gl::TEXTURE_MAG_FILTER, mag);

    // Convert the wrapping mode enum to the raw opengl type
    let (wrap, border) = match sampling.wrap {
        Wrap::ClampToEdge => (gl::CLAMP_TO_EDGE, None),
        Wrap::ClampToBorder(b) => (gl::CLAMP_TO_BORDER, Some(b)),
        Wrap::Repeat => (gl::REPEAT, None),
        Wrap::MirroredRepeat => (gl::MIRRORED_REPEAT, None),
    };

    // Set the wrapping mode (for all 3 axii)
    gl::TextureParameteri(name, gl::TEXTURE_WRAP_S, wrap as i32);
    gl::TextureParameteri(name, gl::TEXTURE_WRAP_T, wrap as i32);
    gl::TextureParameteri(name, gl::TEXTURE_WRAP_R, wrap as i32);

    // Set the border color (if needed)
    if let Some(border) = border {
        gl::TextureParameterfv(name, gl::TEXTURE_BORDER_COLOR, border.as_ptr());
    }
}

// A sampler is used as an interface between textures and Shaders. We can use samplers to read textures within shaders, and each texture has a unique sampler associated with it
// For now, samplers are just simple wrappers around textures. They just help organizing and separating "textures" from actual shader "samplers"
pub struct Sampler<'a, T: Texture>(pub(crate) &'a T);
