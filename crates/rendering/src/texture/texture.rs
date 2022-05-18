use crate::{context::Context, object::{ToGlName, ToGlType, Bind}};
use std::num::NonZeroU32;

use super::TexelLayout;

// Texture filtering type (neartest neighbor vs bilinear)
pub enum MinMagFilter {
    Nearest,
    Bilinear,
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture<T: TexelLayout>: ToGlName + ToGlType + Bind {
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


    // Calculate the number of texels that make up this texture
    fn count_texels(&self) -> u32;

    // Calculate the uncompressed size of the texture
    fn count_bytes(&self) -> u64 {
        u64::from(T::bytes()) * u64::from(self.count_texels())
    }
}
