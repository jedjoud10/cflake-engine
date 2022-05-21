use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
    num::NonZeroU32,
    rc::Rc,
};

use crate::{
    context::Context,
    object::{ToGlName, ToGlType},
};

use super::{TexelLayout, Texture, TextureMode};

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

// Data specifically for bindless textures
pub(crate) struct Bindless {
    // The GPU handle for the texture
    pub(crate) handle: u64,

    // Is the handle resident (does the texutre live on the GPU)?
    pub(crate) resident: Cell<bool>,
}

// A sampler is used as an interface between textures and Shaders. We can use samplers to read textures within shaders, and each texture has a unique sampler associated with it
pub struct Sampler {
    // Name of the texture
    pub(crate) texture: NonZeroU32,

    // The texture's target
    pub(crate) target: u32,

    // Optional bindless handle (since not all textures are bindless textures)
    pub(crate) bindless: Option<Bindless>,
}

// Apply some sampling parameters to a specific texture, and convert it into a sampler object
pub(super) unsafe fn apply(name: NonZeroU32, target: u32, mode: TextureMode, sampling: Sampling) -> Sampler {
    // We do a bit of enum fetching (this is safe) (trust)
    let filter = std::mem::transmute::<Filter, u32>(sampling.filter);

    // Min and mag filters conversion cause OpenGL suxs
    let min = filter as i32;
    let mag = filter as i32;

    // Set the filters
    gl::TextureParameteri(name.get(), gl::TEXTURE_MIN_FILTER, min);
    gl::TextureParameteri(name.get(), gl::TEXTURE_MAG_FILTER, mag);

    // Convert the wrapping mode enum to the raw opengl type
    let (wrap, border) = match sampling.wrap {
        Wrap::ClampToEdge => (gl::CLAMP_TO_EDGE, None),
        Wrap::ClampToBorder(b) => (gl::CLAMP_TO_BORDER, Some(b)),
        Wrap::Repeat => (gl::REPEAT, None),
        Wrap::MirroredRepeat => (gl::MIRRORED_REPEAT, None),
    };

    // Set the wrapping mode (for all 3 axii)
    gl::TextureParameteri(name.get(), gl::TEXTURE_WRAP_S, wrap as i32);
    gl::TextureParameteri(name.get(), gl::TEXTURE_WRAP_T, wrap as i32);
    gl::TextureParameteri(name.get(), gl::TEXTURE_WRAP_R, wrap as i32);

    // Set the border color (if needed)
    if let Some(border) = border {
        gl::TextureParameterfv(name.get(), gl::TEXTURE_BORDER_COLOR, border.as_ptr());
    }

    // Create the bindless handle if we need to use bindles textures
    let bindless = (mode == TextureMode::Dynamic).then(|| Bindless {
        handle: gl::GetTextureHandleARB(name.get()),
        resident: Cell::new(false),
    });

    // Create the sampler object
    Sampler { texture: name, target, bindless }
}
