use std::{marker::PhantomData, num::NonZeroU32, rc::Rc, collections::HashSet, cell::RefCell};

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


// A linked sampler is simply a sampler that is associated with a texture

// A sampler is the interface between Textures and Shaders. Samplers allow us to read textures within shaders
pub struct Sampler {
    // The raw OpenGL name of the underlying sampler object
    name: Rc<NonZeroU32>,

    // The name of the bound texture
    texture: Option<NonZeroU32>,

    // Optional bindless handle of said texture
    handle: Option<u64>
}

impl Drop for Sampler {
    fn drop(&mut self) {
        // If we have no more sampler objects, we must deallocate the last one
        if Rc::strong_count(&self.name) == 1 {
            unsafe {
                gl::DeleteSamplers(1, &self.name.get());
            }
        }
    }
}

impl Sampler {
    // Create a new sampler using some sampling parameters
    pub fn new(filter: Filter, wrap: Wrap, ctx: &mut Context) -> Self {
        // Create a raw sampler object
        let name = unsafe {
            let mut name = 0u32;
            gl::CreateSamplers(1, &mut name);
            NonZeroU32::new(name).unwrap()
        };

        // Set the sampler parameters
        unsafe {
            // We do a bit of enum fetching (this is safe) (trust)
            let filter = std::mem::transmute::<Filter, u32>(filter);

            // Min and mag filters conversion cause OpenGL suxs
            let min = filter as i32;
            let mag = filter as i32;

            // Set the filters
            gl::SamplerParameteri(name.get(), gl::TEXTURE_MIN_FILTER, min);
            gl::SamplerParameteri(name.get(), gl::TEXTURE_MAG_FILTER, mag);

            
            // Convert the wrapping mode enum to the raw opengl type
            let (wrap, border) = match wrap {
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
                gl::SamplerParameterfv(name.get(), gl::TEXTURE_BORDER_COLOR, border.as_ptr());
            }
        }

        // Construct unique sampler object
        Self {
            name: Rc::new(name),
            texture: None,
            handle: None,
        }
    }

    // Clone the sampler, but create a unique bindless handle if we need to
    pub(super) fn clone_unique(&self, mode: TextureMode, name: NonZeroU32) -> Self {
        // Create the bindless handle (if needed)
        let handle = (mode == TextureMode::Dynamic).then(|| unsafe {
            gl::GetTextureSamplerHandleARB(name.get(), self.name.get())
        });

        Self {
            name: self.name.clone(),
            texture: self.texture.clone(),
            handle
        }
    }
}

impl ToGlName for Sampler {
    fn name(&self) -> NonZeroU32 {
        *self.name
    }
}
