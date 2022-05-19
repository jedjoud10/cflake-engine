use super::{TexelLayout, Texture, TextureMode, create_texture_raw};
use crate::{
    context::Cached,
    object::{Active, Bind, ToGlName, ToGlType},
};
use std::{marker::PhantomData, num::{NonZeroU32, NonZeroU8}};

// A 2D texture that will be used for rendering objects
pub struct Texture2D<T: TexelLayout> {
    // Internal OpenGL shit
    texture: NonZeroU32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    levels: u8,

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

impl<T: TexelLayout> Texture for Texture2D<T> {
    type Layout = T;

    type Dimensions = vek::Extent2<u16>;

    type Region = (vek::Vec2<u16>, vek::Extent2<u16>);

    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }

    fn region(&self) -> Self::Region {
        (vek::Vec2::zero(), self.dimensions)
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn count_texels(&self) -> u32 {
        self.dimensions.as_().product()
    }

    fn get_layer(&self, level: u8) -> Option<super::MipLayerRef<Self>> {
        (level < self.levels).then(|| super::MipLayerRef::new(self, level)) 
    }

    fn get_layer_mut(&mut self, level: u8) -> Option<super::MipLayerMut<Self>> {
        (level < self.levels).then(|| super::MipLayerMut::new(self, level)) 
    }

    unsafe fn clear_mip_layer_unchecked(&mut self, ctx: &mut crate::context::Context, level: u8, val: Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        gl::ClearTexSubImage(self.name().get(), level as i32, offset.x, offset.y, 0, extent.w, extent.h, 1, format_, type_, &region as *const Self::Region as _);
    }

    unsafe fn update_mip_layer_unchecked(&mut self, ctx: &mut crate::context::Context, level: u8, ptr: *const Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        gl::TextureSubImage2D(self.name().get(), level as i32, offset.x, offset.y, extent.w, extent.h, format_, type_, ptr as _)
    }

    unsafe fn read_mip_layer_unchecked(&self, ctx: &crate::context::Context, level: u8, out: *mut Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        let buf_size = i32::try_from(extent.as_::<u32>().product() * Self::Layout::bytes()).unwrap();
        gl::GetTextureSubImage(self.name().get(), level as i32, offset.x, offset.y, 0, extent.w, extent.h, 1, format_, type_, buf_size, out as _)
    }

    fn new_unchecked(ctx: &mut crate::context::Context, mode: TextureMode, dimensions: Self::Dimensions, levels: NonZeroU8, data: &[Self::Layout]) -> Option<Self> {
        // Create a new raw OpenGL texture object
        let tex = unsafe {
            create_texture_raw(gl::TEXTURE_2D);
        };

        // Check for mipmaps
        let levels = levels.get();
        let mipmaps = levels > 1;

        // Pre-allocate storage using the texture mode (immutable vs mutable textures)
        unsafe {
            match mode {
                TextureMode::Dynamic | TextureMode::Bindless => gl::TextureStorage2D(tex.get(), levels, T::INTERNAL_FORMAT, size.w as i32, size.h as i32),
                TextureMode::Resizable => gl::TexImage2D(gl::TEXTURE_2D, levels, T::INTERNAL_FORMAT, size.w as i32, size.h as i32, 0, T::FORMAT, gl::UNSIGNED_BYTE, ),
            }
        }


        // Fill the texture with data
        // Make bindless and resident (optional)
        // Create the handle and make it resident
        let handle = gl::GetTextureHandleARB(self.name().get());
        gl::MakeTextureHandleResidentARB(handle);
        handle
    }
}
