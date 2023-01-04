use crate::{Graphics, Texel, Texture};
use std::{marker::PhantomData, num::NonZeroU8};
use vulkan::vk;

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
    min_filter: SamplerFilter,
    mag_filter: SamplerFilter,
    mipmapping: SamplerMipMapping,
}

impl<'a, T: Texture> Sampler<'a, T> {
    // Change the sampler wrapping mode
    pub fn wrapping_mode(self, wrap: SamplerWrap<T::T>) -> Self {
        todo!()
    }

    // Change the sampler minification filter
    pub fn min_filter(self, filter: SamplerFilter) -> Self {
        todo!()
    }

    // Change the sampler magnification filter
    pub fn mag_filter(self, filter: SamplerFilter) -> Self {
        todo!()
    }

    // Change the sampler mipmapping mode
    pub fn mipmap(self, mipmap: SamplerMipMapping) -> Self {
        todo!()
    }
}
