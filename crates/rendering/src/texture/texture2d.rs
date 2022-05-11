use std::{marker::PhantomData, num::NonZeroU32};
use crate::context::Cached;
use super::{TexelLayout, Texture, MinMagFilter};

// A 2D texture that will be used for rendering objects
pub struct Texture2D<T: TexelLayout> {
    // Internal OpenGL shit
    texture: NonZeroU32,

    // Main texture settings
    dimensions: vek::Extent2<u32>,
    mipmaps: bool,
    filter: MinMagFilter,
    
    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: TexelLayout> Cached for Texture2D<T> {}
impl<T: TexelLayout> Texture<T> for Texture2D<T> {
    type Dimensions = vek::Extent2<u32>;

    fn target(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(gl::TEXTURE_2D) }
    }

    fn name(&self) -> NonZeroU32 {
        self.texture
    }

    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }

    fn count_texels(&self) -> u32 {
        self.dimensions.product()
    }
}