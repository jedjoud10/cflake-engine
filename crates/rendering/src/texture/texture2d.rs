use super::{raw::RawTexture, TexelLayout, Texture, TextureMode};
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
    mode: TextureMode,

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

unsafe impl<T: TexelLayout> RawTexture for Texture2D<T> {
    type Layout = T;
    type Dimensions = vek::Extent2<u32>;

    unsafe fn update_mip_layer_unchecked(tex: NonZeroU32, ctx: &mut crate::context::Context, level: u32, ptr: *const std::ffi::c_void, dimensions: Self::Dimensions) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        gl::TextureSubImage2D(tex.get(), level as i32, 0, 0, dimensions.w as i32, dimensions.h as i32, format_, type_, ptr)
    }
}

impl<T: TexelLayout> Texture for Texture2D<T> {
    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn count_texels(&self) -> u32 {
        self.dimensions.product()
    }

    fn get_layer(&self, level: u32) -> Option<super::RefMipLayer<Self>> {
        todo!()
    }

    fn get_layer_mut(&mut self, level: u32) -> Option<super::MutMipLayer<Self>> {
        todo!()
    }
}
