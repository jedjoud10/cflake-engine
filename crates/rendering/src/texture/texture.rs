use crate::{
    context::Context,
    object::{Bind, ToGlName, ToGlType},
};
use std::num::NonZeroU32;
use super::TexelLayout;

// Some settings that tell us exactly how we should generate a texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureMode {
    // Static textures must be set only once, and it is during their initialization
    Static,

    // Dynamic textures can be modified throughout their lifetime, but they cannot change size
    Dynamic,

    // Resizable textures are just dynamic textures that can change their size at will
    Resizable
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture: ToGlName + ToGlType + Bind {
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

    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Calculate the number of texels that make up this texture
    fn count_texels(&self) -> u32;

    // Calculate the uncompressed size of the texture
    fn count_bytes(&self) -> u64 {
        u64::from(Self::Layout::bytes()) * u64::from(self.count_texels())
    }
}
