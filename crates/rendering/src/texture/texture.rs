use crate::context::Context;
use std::num::NonZeroU32;

use super::TexelLayout;

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture<T: TexelLayout> {
    // Textures can have different dimensions
    type Dimensions;

    // The texture's OpenGL target type
    const GL_TARGET: NonZeroU32;

    // Get the texture's name, using a context
    fn name(&self, _ctx: &Context) -> NonZeroU32;

    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Bind the texture so we can modify it
    fn bind(&mut self, _ctx: &mut Context);

    // Calculate the number of texels that make up this texture
    fn count_texels(&self) -> u32;

    // Calculate the uncompressed size of the texture
    fn count_bytes(&self) -> u64 {
        u64::from(T::bytes()) * u64::from(self.count_texels())
    }
}
