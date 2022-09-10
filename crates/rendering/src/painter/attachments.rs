use std::marker::PhantomData;

use crate::{
    context::ToGlName,
    prelude::{DepthTexel, MipLevelMut, Texel, Texture2D, UntypedTexel},
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
#[derive(Clone, Copy)]
pub enum UntypedAttachment {
    TextureLevel {
        texture_name: u32,
        level: u8,
        untyped: UntypedTexel,
    },
}

impl PartialEq for UntypedAttachment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::TextureLevel {
                    texture_name: l_texture_name,
                    level: l_level,
                    untyped: l_untyped,
                },
                Self::TextureLevel {
                    texture_name: r_texture_name,
                    level: r_level,
                    untyped: r_untyped,
                },
            ) => l_texture_name == r_texture_name && l_level == r_level && l_untyped == r_untyped,
        }
    }
}
impl Eq for UntypedAttachment {}

// An attachment is something that we will bind to the painter to be able to render to it
// This attachment trait is just a wrapper around framebuffer attachments
pub trait Attachment<T> {
    fn untyped(&self) -> Option<UntypedAttachment>;
}

// Attachments that use the default texel are disabled
impl Attachment<()> for () {
    fn untyped(&self) -> Option<UntypedAttachment> {
        None
    }
}

// Texture2D mip maps are attachable
impl<'a, T: Texel> Attachment<T> for MipLevelMut<'a, Texture2D<T>> {
    fn untyped(&self) -> Option<UntypedAttachment> {
        Some(UntypedAttachment::TextureLevel {
            texture_name: self.texture().name(),
            level: self.level(),
            untyped: T::untyped(),
        })
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
