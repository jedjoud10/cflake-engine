use std::{marker::PhantomData, num::NonZeroU32};

use crate::{
    context::Context,
    object::{ToGlName, ToGlType},
};

use super::{TexelLayout, Texture};

// Texel filters that are applied to the sampler's mininifcation and magnification parameters
pub enum Filter {
    // Filtering for any texture
    Nearest,
    Linear,

    // Filtering for textures that use mipmaps
    TryMipMapNearest,
    TryMipMapLinear,
}

// Some parameters that we can use to create a new sampler
pub struct SamplerParameters<T: TexelLayout> {
    // Minification and magnification combined
    filter: Filter,

    // Border color
    border: T,
}

impl<T: TexelLayout> Default for SamplerParameters<T> {
    fn default() -> Self {
        Self {
            filter: Filter::Linear,
            border: Default::default(),
        }
    }
}

impl<T: TexelLayout> SamplerParameters<T> {
    // Update the filter of these parameters
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    // Set the border color of these parameters
    pub fn border(mut self, border: T) -> Self {
        self.border = border;
        self
    }
}

// A sampler is the interface between Textures and Shaders. Samplers allow us to read textures within shaders
pub struct Sampler<T: Texture> {
    // The raw OpenGL name of the underlying sampler object
    sampler: NonZeroU32,

    // Unsend and unsync
    _phantom: PhantomData<*const T>,
}

impl<T: Texture> Sampler<T> {
    // Create a new sampler using a texture
    pub fn new(texture: &T, params: SamplerParameters<T::Layout>, ctx: &mut Context) -> Self {
        // Create a raw sampler object
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateSamplers(1, &mut name);
            NonZeroU32::new(name).unwrap()
        };

        // Set the sampler parameters

        Self {
            sampler: name,
            _phantom: Default::default(),
        }
    }
}

impl<T: Texture> ToGlName for Sampler<T> {
    fn name(&self) -> NonZeroU32 {
        self.sampler
    }
}
