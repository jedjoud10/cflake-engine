use assets::Asset;

use super::{ImageTexel, MipMaps, Region, Sampling, Texel, Texture, TextureMode};
use crate::{
    context::Context,
    object::{ToGlName, ToGlTarget},
};
use std::{marker::PhantomData, num::NonZeroU8, rc::Rc};

// A 2D texture that contains multiple pixels that have their own channels
// Each pixel can be either a single value, RG, RGB, or even RGBA
// These individual pixels are called texels, since they are used within the texture
pub struct Texture2D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    levels: NonZeroU8,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for Texture2D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for Texture2D<T> {
    fn target() -> u32 {
        gl::TEXTURE_2D
    }
}

impl<T: Texel> Texture for Texture2D<T> {
    type Region = (vek::Vec2<u16>, vek::Extent2<u16>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as super::Region>::E {
        self.dimensions
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn levels(&self) -> NonZeroU8 {
        self.levels
    }

    fn get_layer(&self, level: u8) -> Option<super::MipLayerRef<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerRef::new(self, level))
    }

    fn get_layer_mut(&mut self, level: u8) -> Option<super::MipLayerMut<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerMut::new(self, level))
    }

    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as super::Region>::E,
        mode: TextureMode,
        levels: NonZeroU8,
    ) -> Self {
        Self {
            name,
            dimensions,
            mode,
            levels,
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
        gl::BindTexture(gl::TEXTURE_2D, 0);
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

impl<'a, T: ImageTexel> Asset<'a> for Texture2D<T> {
    type Args = (&'a mut Context, Sampling, MipMaps, TextureMode);

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let image = image::load_from_memory(data.bytes()).unwrap();
        let image = image.flipv();
        let dimensions = vek::Extent2::new(image.width() as u16, image.height() as u16);
        let texels = T::to_image_texels(image);
        Self::new(
            args.0,
            args.3,
            dimensions,
            args.1,
            args.2,
            texels.as_slice(),
        )
        .unwrap()
    }
}
