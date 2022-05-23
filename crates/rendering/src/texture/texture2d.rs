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

impl<T: TexelLayout> Texture for Texture2D<T> {
    type Layout = T;
    type TexelRegion = (vek::Vec2<u16>, vek::Extent2<u16>);

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
    
    unsafe fn from_raw_parts(name: NonZeroU32, dimensions: <Self::TexelRegion as super::Region>::E, mode: TextureMode, levels: NonZeroU8, bindless: Option<Rc<Bindless>>) -> Self {
        Self { texture: name, dimensions, mode, levels, bindless, _phantom: Default::default() }
    }
}