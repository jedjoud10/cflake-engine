use std::marker::PhantomData;

use crate::{
    context::ToGlName,
    prelude::{
        Depth, DepthTexel, Element, MipLevelMut, MipLevelRef, MultiLayerTexture, Region,
        SingleLayerTexture, Stencil, StencilTexel, Texel, Texture, Texture2D, UntypedTexel,
    },
};

// This is the target for a specific framebuffer attachment
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum AttachmentLocation {
    Color(u32),
    Depth,
    Stencil,
}

// This is a wrapper around framebuffer attachments
// These values will be stored within the canvas
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum UntypedAttachment {
    TextureLevel {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel,
        writable: bool,
    },

    TextureLevelLayer {
        texture_name: u32,
        level: u8,
        layer: u16,
        untyped: UntypedTexel,
        writable: bool,
    },
}

impl UntypedAttachment {
    // Check if the attachment is writable or not
    pub fn writable(&self) -> bool {
        match self {
            UntypedAttachment::TextureLevel { writable, .. }
            | UntypedAttachment::TextureLevelLayer { writable, .. } => *writable,
        }
    }

    // Get the untyped attachment texel
    pub fn texel(&self) -> UntypedTexel {
        match self {
            UntypedAttachment::TextureLevel { untyped, .. }
            | UntypedAttachment::TextureLevelLayer { untyped, .. } => *untyped,
        }
    }
}

// A possible texel type, or simply the unit tuple
pub trait MaybeTexel {}
impl<T: Texel> MaybeTexel for T {}
impl MaybeTexel for () {}

// This trait is implemented for color texels exclusively and the unit tuple
pub trait MaybeColorLayout {}
impl MaybeColorLayout for () {}

// This trait is implemented for depth texels exclusively and the unit tuple
pub trait MaybeDepthTexel: MaybeTexel {}
impl MaybeDepthTexel for () {}
impl<E: Element> MaybeDepthTexel for Depth<E> where Self: DepthTexel {}

// This trait is implemented for stencil texels exclusively and the unit tuple
pub trait MaybeStencilTexel: MaybeTexel {}
impl MaybeStencilTexel for () {}
impl<E: Element> MaybeStencilTexel for Stencil<E> where Self: StencilTexel {}

// A target is a simple abstraction around FBO attachments
// We can get targets from MipLayers and whole Textures if we wish to
// Sometimes these targets are used only for reading, in which case the "writable" field would be set to false
pub struct Target<T: MaybeTexel, A> {
    pub(super) untyped: UntypedAttachment,
    _phantom: PhantomData<(T, A)>,
}

// Gonna need an untyped target for color layouts
pub struct UntypedTarget {
    pub(super) untyped: UntypedAttachment,
}

pub trait AsTarget<T: MaybeTexel> {
    type A;
    fn as_target(self) -> Option<Target<T, Self::A>>;
    fn as_untyped_target(self) -> Option<UntypedTarget>;
}

impl<T: Texel, A> AsTarget<T> for Target<T, A> {
    type A = A;
    fn as_target(self) -> Option<Target<T, Self::A>> {
        Some(self)
    }
    fn as_untyped_target(self) -> Option<UntypedTarget> {
        Some(UntypedTarget {
            untyped: self.untyped,
        })
    }
}

impl AsTarget<()> for () {
    type A = ();
    fn as_target(self) -> Option<Target<(), Self::A>> {
        None
    }
    fn as_untyped_target(self) -> Option<UntypedTarget> {
        None
    }
}

// Implemented for object that have a single layer and that be converted to simple targets
pub trait SingleLayerIntoTarget<T: Texel>: Sized {
    fn target(self) -> Target<T, Self>;
}

// Convert single layered mip level into a writable/readable target
impl<'a, T: SingleLayerTexture> SingleLayerIntoTarget<T::T> for MipLevelMut<'a, T> {
    fn target(self) -> Target<T::T, Self> {
        assert!(self.texture().mode().write_permission(), "Cannot use a MipLevelMut as a writable painter target because the texture is static");

        Target {
            untyped: UntypedAttachment::TextureLevel {
                texture_name: self.texture().name(),
                level: self.level(),
                untyped: <T::T as Texel>::untyped(),
                writable: true,
            },
            _phantom: PhantomData,
        }
    }
}

// Convert single layered mip level into a readable target
impl<'a, T: SingleLayerTexture> SingleLayerIntoTarget<T::T> for MipLevelRef<'a, T> {
    fn target(self) -> Target<T::T, Self> {
        Target {
            untyped: UntypedAttachment::TextureLevel {
                texture_name: self.texture().name(),
                level: self.level(),
                untyped: <T::T as Texel>::untyped(),
                writable: false,
            },
            _phantom: PhantomData,
        }
    }
}

// This looks cursed but it basically allows us to not have to write ".target" every time we wish to use a single layered texture
impl<'a, T: Texture> AsTarget<T::T> for MipLevelMut<'a, T>
where
    Self: SingleLayerIntoTarget<T::T>,
{
    type A = MipLevelMut<'a, T>;
    fn as_target(self) -> Option<Target<T::T, Self::A>> {
        Some(self.target())
    }
    fn as_untyped_target(self) -> Option<UntypedTarget> {
        Some(UntypedTarget {
            untyped: self.target().untyped,
        })
    }
}

// Non writable AsTarget
impl<'a, T: Texture> AsTarget<T::T> for MipLevelRef<'a, T>
where
    Self: SingleLayerIntoTarget<T::T>,
{
    type A = MipLevelRef<'a, T>;
    fn as_target(self) -> Option<Target<T::T, Self::A>> {
        Some(self.target())
    }
    fn as_untyped_target(self) -> Option<UntypedTarget> {
        Some(UntypedTarget {
            untyped: self.target().untyped,
        })
    }
}

// Implemented for objects that have multiple layers and that must use a specific when when fetching targets
// Only used for cubemaps, bundled texture 2d, and texture 3ds
pub trait MultilayerIntoTarget<T: Texel>: Sized {
    fn target(self, layer: u16) -> Option<Target<T, Self>>;
}

// Convert multi layered mip levels into a writable/readable target
impl<'a, T: MultiLayerTexture> MultilayerIntoTarget<T::T> for MipLevelMut<'a, T> {
    fn target(self, layer: u16) -> Option<Target<T::T, Self>> {
        T::is_layer_valid(self.texture(), layer).then(|| Target {
            untyped: UntypedAttachment::TextureLevelLayer {
                texture_name: self.texture().name(),
                level: self.level(),
                layer,
                untyped: <T::T as Texel>::untyped(),
                writable: true,
            },
            _phantom: PhantomData,
        })
    }
}

// Convert multi layered mip levels into a readable target
impl<'a, T: MultiLayerTexture> MultilayerIntoTarget<T::T> for MipLevelRef<'a, T> {
    fn target(self, layer: u16) -> Option<Target<T::T, Self>> {
        T::is_layer_valid(self.texture(), layer).then(|| Target {
            untyped: UntypedAttachment::TextureLevelLayer {
                texture_name: self.texture().name(),
                level: self.level(),
                layer,
                untyped: <T::T as Texel>::untyped(),
                writable: false,
            },
            _phantom: PhantomData,
        })
    }
}

// This is implemented for all tuples that contain types of attachments of the specifici painter color layout
pub trait ColorTupleTargets<C: MaybeColorLayout> {
    fn untyped_targets(self) -> Option<Vec<UntypedTarget>>;
}
