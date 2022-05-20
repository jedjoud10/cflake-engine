use std::{marker::PhantomData, num::NonZeroU32};

use crate::{
    context::Context,
    object::{ToGlName, ToGlType},
};

use super::{TexelLayout, Texture};

// Texel filters that are applied to the sampler's mininifcation and magnification parameters
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
pub enum Wrap<T: TexelLayout> {
    // Oop sorry no more custom discriminent :(
    ClampToEdge,
    ClampToBorder(T),
    Repeat,
    MirroredRepeat,
}

// Some parameters that we can use to create a new sampler
pub struct SamplerParameters<T: TexelLayout> {
    // Minification and magnification combined
    filter: Filter,

    // T and R wrapping modes
    wrap: Wrap<T>,
}

impl<T: TexelLayout> Default for SamplerParameters<T> {
    fn default() -> Self {
        Self {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
        }
    }
}

impl<T: TexelLayout> SamplerParameters<T> {
    // Update the filter of these parameters
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    // Set the wrapping mode
    pub fn wrap(mut self, wrap: Wrap<T>) -> Self {
        self.wrap = wrap;
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
        unsafe {
            // We do a bit of enum fetching (this is safe) (trust)
            let filter = std::mem::transmute::<Filter, u32>(params.filter);

            // Min and mag filters conversion cause OpenGL suxs
            let min = filter as i32;
            let mag = filter as i32;

            // Set the filters
            gl::SamplerParameteri(name.get(), gl::TEXTURE_MIN_FILTER, min);
            gl::SamplerParameteri(name.get(), gl::TEXTURE_MAG_FILTER, mag);

            
            // Convert the wrapping mode enum to the raw opengl type
            let (wrap, border) = match params.wrap {
                Wrap::ClampToEdge => (gl::CLAMP_TO_EDGE, None),
                Wrap::ClampToBorder(b) => (gl::CLAMP_TO_BORDER, Some(b)),
                Wrap::Repeat => (gl::REPEAT, None),
                Wrap::MirroredRepeat => (gl::MIRRORED_REPEAT, None),
            };
            
            // Set the wrapping mode (for all 3 axii)
            gl::SamplerParameteri(name.get(), gl::TEXTURE_WRAP_S, wrap as i32);
            gl::SamplerParameteri(name.get(), gl::TEXTURE_WRAP_T, wrap as i32);
            gl::SamplerParameteri(name.get(), gl::TEXTURE_WRAP_R, wrap as i32);
            
            // Set the border color (if needed)
            if let Some(border) = border {
                // TODO: Check if this actually works
                gl::SamplerParameterfv(name.get(), gl::TEXTURE_BORDER_COLOR, &border as *const T::Layout as *const f32);
            }
        }

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
