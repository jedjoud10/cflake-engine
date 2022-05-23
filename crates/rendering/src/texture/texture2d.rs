use super::{Bindless, TexelLayout, Texture, TextureMode};
use crate::{
    context::Cached,
    object::{Active, Bind, ToGlName, ToGlType},
};
use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU8},
    ptr::{null, NonNull},
    rc::Rc,
};

// A 2D texture that will be used for rendering objects
pub struct Texture2D<T: TexelLayout> {
    // Internal OpenGL shit
    texture: NonZeroU32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    levels: NonZeroU8,
    bindless: Option<Rc<Bindless>>,

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
    unsafe fn bind_raw_unchecked(&mut self, ctx: &mut crate::context::Context) {
        gl::BindTexture(self.target(), self.name().get())
    }
}

/*
unsafe impl<T: TexelLayout> TextureAllocator for Texture2D<T> {
    unsafe fn alloc_immutable_storage(name: NonZeroU32, levels: u8, dimensions: Self::Dimensions) {
        gl::TextureStorage2D(name.get(), levels as i32, Self::Layout::INTERNAL_FORMAT, dimensions.w as i32, dimensions.h as i32);
    }

    unsafe fn alloc_resizable_storage(name: NonZeroU32, level: u8, dimensions: Self::Dimensions, ptr: *const Self::Layout) {
        gl::BindTexture(gl::TEXTURE_2D, name.get());
        gl::TexImage2D(
            gl::TEXTURE_2D,
            level as i32,
            Self::Layout::INTERNAL_FORMAT as i32,
            dimensions.w as i32,
            dimensions.h as i32,
            0,
            Self::Layout::FORMAT,
            gl::UNSIGNED_BYTE,
            ptr as _,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    unsafe fn update_sub_region(name: NonZeroU32, level: u8, region: Self::Region, ptr: *const Self::Layout) {
        gl::TextureSubImage2D(
            name.get(),
            level as i32,
            region.0.x as i32,
            region.0.y as i32,
            region.1.w as i32,
            region.1.h as i32,
            Self::Layout::FORMAT,
            gl::UNSIGNED_BYTE,
            ptr as _,
        );
    }

    unsafe fn from_raw_parts(name: NonZeroU32, dimensions: Self::Dimensions, mode: TextureMode, levels: NonZeroU8, bindless: Option<Rc<Bindless>>) -> Self {
        Self {
            texture: name,
            dimensions,
            mode,
            levels,
            bindless,
            _phantom: Default::default(),
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

    fn sampler(&self) -> super::Sampler<Self> {
        super::Sampler(self)
    }

    fn bindless(&self) -> Option<&Bindless> {
        self.bindless.as_ref().map(Rc::as_ref)
    }

    fn get_layer(&self, level: u8) -> Option<super::MipLayerRef<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerRef::new(self, level))
    }

    fn get_layer_mut(&mut self, level: u8) -> Option<super::MipLayerMut<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerMut::new(self, level))
    }

    fn dimensions_to_region_at_origin(dimensions: Self::Dimensions) -> Self::Region {
        (vek::Vec2::zero(), dimensions)
    }
}
*/