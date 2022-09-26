

use std::marker::PhantomData;

use crate::{
    context::ToGlName,
    prelude::{MipLevelMut, Texel, Texture2D, UntypedTexel, Region, SingleLayerTexture, MultiLayerTexture},
};

use super::{PainterColorLayout, PainterDepthTexel, PainterStencilTexel};

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
pub enum UntypedAttachment {
    TextureLevel {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel,
    },

    TextureLevelLayer {
        texture_name: u32,
        level: u8,
        layer: u16,
        untyped: UntypedTexel,
    }
}

// A target is a simple abstraction around FBO attachments
// We can get targets from MipLayers and whole Textures if we wish to
pub struct Target<'a, A> {
    pub(crate) untyped: UntypedAttachment,
    object: &'a mut A,
}


// Implemented for object that have a single layer and that be converted to simple targets
pub trait SingleLayerIntoTarget: Sized {
    fn target(&mut self) -> Target<Self>;
}

// Convert single layered mip levels into a target
impl<'a, T: SingleLayerTexture> SingleLayerIntoTarget for MipLevelMut<'a, T> {
    fn target(&mut self) -> Target<Self> {
        Target {
            untyped: UntypedAttachment::TextureLevel { texture_name: self.texture().name(), level: self.level(), untyped: <T::T as Texel>::untyped() },
            object: self,
        }
    }
}

// Implemented for objects that have multiple layers and that must use a specific when when fetching targets
// Only used for cubemaps, bundled texture 2d, and texture 3ds
pub trait MultilayerIntoTarget: Sized {
    fn target(&mut self, layer: u16) -> Option<Target<Self>>;
}

// Convert multi layered mip levels into a target
impl<'a, T: MultiLayerTexture> MultilayerIntoTarget for MipLevelMut<'a, T> {
    fn target(&mut self, layer: u16) -> Option<Target<Self>> {
        T::is_layer_valid(self.texture(), layer).then(|| Target {
            untyped: UntypedAttachment::TextureLevelLayer { texture_name: self.texture().name(), level: self.level(), layer, untyped: <T::T as Texel>::untyped() },
            object: self,
        })
    }
}

// Attachments are something that we can bind to textures
pub trait Attachment<T> {
    fn untyped(&self) -> Option<UntypedAttachment>;
}

// Attachments that use the default texel are disabled

impl Attachment<()> for () {
    fn untyped(&self) -> Option<UntypedAttachment> {
        None
    }
}

// This is implemented for all tuples that contain types of attachments of the specifici painter color layout
pub trait ColorAttachmentLayout<C: PainterColorLayout> {
    fn untyped(&self) -> Option<Vec<UntypedAttachment>>;
}
impl ColorAttachmentLayout<()> for () {
    fn untyped(&self) -> Option<Vec<UntypedAttachment>> {
        None
    }
}

// TODO: Simplify this a tiny bit I guess?
// This is implemented for all attachments that use this painter depth texel
pub trait DepthAttachment<D: PainterDepthTexel>: Attachment<D> {}
impl<D: PainterDepthTexel + Texel, A: Attachment<D>> DepthAttachment<D> for A {}
impl DepthAttachment<()> for () {}

// This is implemented for all attachments that use this painter stencil texel
pub trait StencilAttachment<S: PainterStencilTexel>: Attachment<S> {}
impl<S: PainterStencilTexel + Texel, A: Attachment<S>> StencilAttachment<S> for A {}
impl StencilAttachment<()> for () {}
