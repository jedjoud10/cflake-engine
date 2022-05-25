use super::{Bindless, TexelLayout, Texture, TextureMode, TextureAllocator, Region};
use crate::{
    context::Cached,
    object::{Name, Target},
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
    texture: Name,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    levels: NonZeroU8,
    bindless: Option<Rc<Bindless>>,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: TexelLayout> Cached for Texture2D<T> {}

impl<T: TexelLayout> 

impl<T: TexelLayout> ToGlName for Texture2D<T> {
    fn name(&self) -> &Name {
        &self.texture
    }
}

impl<T: TexelLayout> ToGlType for Texture2D<T> {
    fn target(&self) -> &Target {
        Target::from(gl::TEXTURE_2D)
    }
}

impl<T: TexelLayout> TextureAllocator for Texture2D<T> {
    type TexelRegion = (vek::Vec2<u16>, vek::Extent2<u16>);

    unsafe fn alloc_immutable_storage(name: u32, extent: <Self::TexelRegion as Region>::E, levels: u8, ptr: *const std::ffi::c_void) {
        gl::TextureStorage2D(name, levels as i32, T::INTERNAL_FORMAT, extent.w as i32, extent.h as i32);
        gl::TextureSubImage2D(name, 0, 0, 0, extent.w as i32, extent.h as i32, T::FORMAT, T::TYPE, ptr);
    }

    unsafe fn alloc_resizable_storage(name: u32, extent: <Self::TexelRegion as Region>::E, unique_level: u8, ptr: *const std::ffi::c_void) {
        gl::BindTexture(gl::TEXTURE_2D, name);
        gl::TexImage2D(gl::TEXTURE_2D, unique_level as i32, T::INTERNAL_FORMAT as i32, extent.w as i32, extent.h as i32, 0, T::FORMAT, T::TYPE, ptr);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    unsafe fn update_subregion(name: u32, region: Self::TexelRegion, ptr: *const std::ffi::c_void) {
        let origin = region.origin();
        let extent = region.extent();
        gl::TextureSubImage2D(name, 0, origin.x as i32, origin.y as i32, extent.w as i32, extent.h as i32, T::FORMAT, T::TYPE, ptr);
    }
}

impl<T: TexelLayout> Texture for Texture2D<T> {
    type Layout = T;
    

    fn dimensions(&self) -> <Self::TexelRegion as super::Region>::E {
        self.dimensions
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
    
    unsafe fn from_raw_parts(name: u32, dimensions: <Self::TexelRegion as super::Region>::E, mode: TextureMode, levels: NonZeroU8, bindless: Option<Rc<Bindless>>) -> Self {
        Self { texture: Name::from(name), dimensions, mode, levels, bindless, _phantom: Default::default() }
    }
}