use std::{num::NonZeroU8, marker::PhantomData};

use world::{Handle, UntypedHandle, Storage};
use crate::{canvas::Canvas, object::{ToGlName, ToGlTarget}};
use super::{Texture2D, Texel, TexelFormat, Texture, TextureMode, Region};

// A 2D render target texture that we will draw to using a canvas
// Render targets can be used as normal textures, but they cannot be loaded in
pub struct RenderTarget2D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for RenderTarget2D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for RenderTarget2D<T> {
    fn target() -> u32 {
        gl::TEXTURE_2D
    }
}

impl<T: Texel> Texture for RenderTarget2D<T> {
    type Region = (vek::Vec2<u16>, vek::Extent2<u16>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as super::Region>::E {
        self.dimensions
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn levels(&self) -> NonZeroU8 {
        NonZeroU8::new(1).unwrap()
    }

    fn get_layer(&self, level: u8) -> Option<super::MipLayerRef<Self>> {
        (level == 0).then(|| super::MipLayerRef::new(self, level))
    }

    fn get_layer_mut(&mut self, level: u8) -> Option<super::MipLayerMut<Self>> {
        (level == 0).then(|| super::MipLayerMut::new(self, level))
    }

    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as super::Region>::E,
        mode: TextureMode,
        levels: NonZeroU8,
    ) -> Self {
        assert_eq!(levels.get(), 0);
        Self {
            name,
            dimensions,
            mode,
            _phantom: Default::default(),
        }
    }

    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        levels: u8,
        ptr: *const std::ffi::c_void,
    ) {
        gl::TextureStorage2D(
            name,
            levels as i32,
            T::INTERNAL_FORMAT,
            extent.w as i32,
            extent.h as i32,
        );
        gl::TextureSubImage2D(
            name,
            0,
            0,
            0,
            extent.w as i32,
            extent.h as i32,
            T::FORMAT,
            T::TYPE,
            ptr,
        );
    }

    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const std::ffi::c_void,
    ) {
        gl::BindTexture(gl::TEXTURE_2D, name);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            unique_level as i32,
            T::INTERNAL_FORMAT as i32,
            extent.w as i32,
            extent.h as i32,
            0,
            T::FORMAT,
            T::TYPE,
            ptr,
        );
    }

    unsafe fn update_subregion(name: u32, region: Self::Region, ptr: *const std::ffi::c_void) {
        let origin = region.origin();
        let extent = region.extent();
        gl::TextureSubImage2D(
            name,
            0,
            origin.x as i32,
            origin.y as i32,
            extent.w as i32,
            extent.h as i32,
            T::FORMAT,
            T::TYPE,
            ptr,
        );
    }
}