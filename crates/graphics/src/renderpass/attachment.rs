use std::marker::PhantomData;

use crate::{
    ColorLayout, ColorTexel, DepthElement, DepthStencilLayout,
    LoadOp, Stencil, StoreOp, Texel, Texture, Texture2D,
};

// A color attachment that is passed to the render pass when starting it
pub trait ColorAttachments<'a, C: ColorLayout> {
}

// A depth stencil attachment that is passed to the render pass when starting it
pub trait DepthStencilAttachment<'a, DS: DepthStencilLayout> {}
impl<'a> DepthStencilAttachment<'a, ()> for () {}

// A render target that can be used inside a renderpass (attachment)
pub struct RenderTarget<'a, T: Texel> {
    _phantom: PhantomData<T>,
    _phantom2: PhantomData<&'a ()>,
}

impl<'a, T: Texel> RenderTarget<'a, T> {
}

impl<'a, T: ColorTexel> ColorAttachments<'a, T>
    for RenderTarget<'a, T>
{
}
