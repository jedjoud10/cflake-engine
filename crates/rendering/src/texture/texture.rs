use crate::{
    context::Context,
    object::{Bind, ToGlName, ToGlType},
};
use std::{num::NonZeroU32, marker::PhantomData};
use self::raw::RawTexture;

use super::TexelLayout;

// Some settings that tell us exactly how we should generate a texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureMode {
    // Static textures must be set only once, and it is during their initialization
    Static,

    // Dynamic textures can be modified throughout their lifetime, but they cannot change size
    Dynamic,

    // Resizable textures are just dynamic textures that can change their size at will
    Resizable,

    // A bindless static texture
    BindlessStatic,

    // A bindless dynamic texture
    BindlessDynamic,
}

// An immutable mip layer that we can use to read from the texture
pub struct RefMipLayer<'a, T: Texture> {
    // El texture
    texture: &'a T,
    
    // The level of the mip layer
    level: u32,

    // The dimensions of the specific mip layer
    dimensions: T::Dimensions,
    _phantom: PhantomData<*const ()>,
}

// A mutable mip layer that we can use to write to the texture
pub struct MutMipLayer<'a, T: Texture> {
    // El texture
    texture: &'a mut T,
    
    // The level of the mip layer
    level: u32,

    // The dimensions of the specific mip layer
    dimensions: T::Dimensions,
    _phantom: PhantomData<*const ()>,
}


// Raw texture stuff, like allocations and generation
pub(super) mod raw {
    use std::{num::NonZeroU32, ffi::c_void};

    use crate::{object::{ToGlName, ToGlType, Bind}, context::Context, texture::TexelLayout};

    use super::TextureMode;
    // Very raw texture indeed
    pub unsafe trait RawTexture: ToGlName + ToGlType + Bind + Sized {
        // Output texel layout
        type Layout: TexelLayout;

        // Textures can have different dimensions
        type Dimensions;  

        // Create a new raw OpenGL texture object
        unsafe fn gen_gl_tex(ctx: &mut Context) -> NonZeroU32 {
            let mut tex = 0u32;
            gl::GenTextures(1, &mut tex);
            NonZeroU32::new(tex).unwrap()
        }
    
        // Fetch the bindless texture handle for this texture if possible
        unsafe fn gen_gl_bindless_handle(&mut self, ctx: &mut Context, mode: TextureMode) -> Option<u64> {
            // Check if the texture is even valid
            (TextureMode::BindlessStatic == mode || TextureMode::BindlessDynamic == mode).then(|| {
                // Create the handle and make it resident
                let handle = gl::GetTextureHandleARB(self.name().get());
                gl::MakeTextureHandleResidentARB(handle);
                handle
            })
        }

        // Update the contents of a whole single mip layer without checking anything
        unsafe fn update_mip_layer_unchecked(tex: NonZeroU32, ctx: &mut Context, level: u32, ptr: *const c_void, dimensions: Self::Dimensions);
    }
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture: RawTexture {
    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Calculate the number of texels that make up this texture
    fn count_texels(&self) -> u32;

    // Get a single mip level from the texture, immutably
    fn get_layer(&self, level: u32) -> Option<RefMipLayer<Self>>; 

    // Get a single mip level from the texture, mutably
    fn get_layer_mut(&mut self, level: u32) -> Option<MutMipLayer<Self>>;

    // Calculate the uncompressed size of the texture
    fn count_bytes(&self) -> u64 {
        u64::from(Self::Layout::bytes()) * u64::from(self.count_texels())
    }
}
