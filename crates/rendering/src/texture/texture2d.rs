use super::{MinMagFilter, TexelLayout, Texture};
use crate::{
    context::Cached,
    object::{Active, Bind, ToGlName, ToGlType},
};
use std::{marker::PhantomData, num::NonZeroU32};

// A 2D texture that will be used for rendering objects
pub struct Texture2D<T: TexelLayout> {
    // Internal OpenGL shit
    texture: NonZeroU32,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: TexelLayout> Cached for Texture2D<T> {}

impl<T: TexelLayout> ToGlName for Texture2D<T> {
    fn name(&self) -> NonZeroU32 {
        self.texture
    }
}

impl<T: TexelLayout> ToGlType for Texture2D<T> {
    fn target(&self) -> u32 {
        gl::TEXTURE_2D
    }
}

impl<T: TexelLayout> Bind for Texture2D<T> {
    fn bind(&mut self, _ctx: &mut crate::context::Context, function: impl FnOnce(crate::object::Active<Self>)) {
        unsafe {
            let target = self.target();
            gl::BindTexture(target, self.target());
            function(Active::new(self, _ctx));
        }
    }
}

impl<T: TexelLayout> Texture<T> for Texture2D<T> {
    type Dimensions = vek::Extent2<u32>;

    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }

    fn count_texels(&self) -> u32 {
        self.dimensions.product()
    }
}
