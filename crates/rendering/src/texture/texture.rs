use crate::context::Context;
use std::num::NonZeroU32;

use super::TexelLayout;

// Texture filtering type (neartest neighbor vs bilinear)
pub enum MinMagFilter {
    Nearest, Bilinear
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture<T: TexelLayout> {
    // Textures can have different dimensions
    type Dimensions;

    // Create a new raw OpenGL texture object
    unsafe fn gen_gl_tex(ctx: &mut Context) -> NonZeroU32 {
        let mut tex = 0u32;
        gl::GenTextures(1, &mut tex);
        NonZeroU32::new_unchecked(tex)
    }

    // Get the texture's target, as a function
    fn target(&self) -> NonZeroU32;

    // Get the texture's OpenGL ID name
    fn name(&self) -> NonZeroU32;

    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Bind the texture so we can modify it
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(&Self, u32)) {
        unsafe {
            let target = self.target().get();
            gl::BindTexture(target, self.target().get());
            function(self, self.target().get());
        }
    }

    // Calculate the number of texels that make up this texture
    fn count_texels(&self) -> u32;

    // Calculate the uncompressed size of the texture
    fn count_bytes(&self) -> u64 {
        u64::from(T::bytes()) * u64::from(self.count_texels())
    }
}
