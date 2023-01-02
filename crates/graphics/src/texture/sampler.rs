use std::{marker::PhantomData, num::NonZeroU8};
use vulkan::vk;
use crate::{Texel, Graphics, Texture};

// Texel filters that are applied to the samplers's mininifcation and magnification parameters
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerFilter {
    Nearest,
    Linear,
}

// Wrapping mode utilized by the sampler address mode
#[derive(Clone, Copy, PartialEq)]
pub enum SamplerWrap<T: Texel> {
    ClampToEdge,
    ClampToBorder(T::Storage),
    Repeat,
    MirroredRepeat,
}

// Mipmapping config for texture samplers within shaders
pub enum SamplerMipMapping {
    Disabled,

    /*
    TODO: Implement mipmapping and anisotropy
    mipmap_lod_bias: f32,
    mipmap_lod_range: (f32, f32),
    mipmap_aniso_samples: Option<NonZeroU8>,
    */
    Enabled,
}

// A sampler is a special objects that allows us to read textures from within shaders
// We might reuse samplers when calling try_fetch_sampler with the same parameters  
pub struct Sampler<'a, T: Texture> {
    texture: &'a T,
    wrap: SamplerWrap<T::T>,
    filter: SamplerFilter,
    mipmapping: SamplerMipMapping
}