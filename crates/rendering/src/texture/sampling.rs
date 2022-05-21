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

// Some unique data that will be specifically valid for bindless textures
pub struct Bindless {
    // The GPU handle for the texture
    pub(crate) handle: u64,

    // Is the handle resident (does the texutre live on the GPU)?
    pub(crate) resident: Cell<bool>,

    // Residency time-out that will be used to automatically disable residency if the texture sampler isn't used as much
    pub(crate) timeout: Duration,

    // The last time this bindless texture's sampelr was used
    pub(crate) last: Cell<Instant>,
}

impl Drop for Bindless {
    fn drop(&mut self) {
        // If we drop a bindless handle, we must make it non-resident
        unsafe {
            self.resident.set(false);
            gl::MakeTextureHandleNonResidentARB(self.handle);
        }
    }
}

impl Bindless {
    // Get the bindless handle
    pub fn handle(&self) -> u64 {
        self.handle
    }

    // Get the current residency state
    pub fn is_resident(&self) -> bool {
        self.resident.get()
    }

    // Get the current time-out value
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    // Get the last time we used the bindless sampler
    pub fn last(&self) -> Instant {
        self.last.get()
    }
}

// A sampler is used as an interface between textures and Shaders. We can use samplers to read textures within shaders, and each texture has a unique sampler associated with it
pub struct Sampler {
    // Name of the texture
    pub(crate) texture: NonZeroU32,

    // The texture's target
    pub(crate) target: u32,

    // Optional bindless handle (since not all textures are bindless textures)
    pub(crate) bindless: Option<Rc<Bindless>>,
}

// Apply some sampling parameters to a specific texture, and convert it into a sampler object
pub(super) unsafe fn apply(ctx: &mut Context, name: NonZeroU32, target: u32, mode: TextureMode, sampling: Sampling) -> Sampler {
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
    let bindless = (mode == TextureMode::Dynamic).then(|| {
        // Create the RC first
        let rc = Rc::new(Bindless {
            handle: gl::GetTextureHandleARB(name.get()),
            resident: Cell::new(false),

            // TODO: Handle custom values for timeout residency
            timeout: Duration::from_millis(200),
            last: Cell::new(Instant::now()),
        });

        // Then clone it to be able to store it within the context
        ctx.bindless.push(rc.clone());

        // Boink
        rc
    });

    // Create the sampler object
    Sampler { texture: name, target, bindless }
}
