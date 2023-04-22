use std::marker::PhantomData;

use thiserror::Error;
use vek::Slerp;

use crate::{
    ColorLayout, ColorTexel, Depth, DepthElement, DepthStencilLayout, LoadOp, Stencil,
    StencilElement, StoreOp, Texel, Texture, Texture2D,
};

// A color attachment that is passed to the render pass when starting it
pub trait ColorAttachments<'a, C: ColorLayout>: 'a {
    // Get the texture views that we will render to
    fn views(&self) -> Vec<&'a wgpu::TextureView>;
}

// A depth stencil attachment that is passed to the render pass when starting it
pub trait DepthStencilAttachment<'a, DS: DepthStencilLayout>: 'a {
    fn view(&self) -> Option<&'a wgpu::TextureView>;
}
impl<'a> DepthStencilAttachment<'a, ()> for () {
    fn view(&self) -> Option<&'a wgpu::TextureView> {
        None
    }
}

// Trait implemented for types that can be converted to render targets
pub trait AsRenderTarget<'a, T: Texel> {
    type Error: std::error::Error;

    // Get the inner texture view (if valid)
    fn try_get_view(&self) -> Result<wgpu::TextureView, Self::Error>;

    // Try to convert self into a render target
    fn as_render_target(&self) -> Result<RenderTarget<'a, T>, Self::Error>;
}

// A render target that can be used inside a renderpass (attachment)
pub struct RenderTarget<'a, T: Texel> {
    pub(crate) _phantom: PhantomData<T>,
    pub(crate) view: &'a wgpu::TextureView,
}

impl<'a, T: Texel> RenderTarget<'a, T> {
    // Get the raw texture view that we will write to
    pub fn view(&self) -> &'a wgpu::TextureView {
        self.view
    }
}

impl<'a> ColorAttachments<'a, ()> for () {
    fn views(&self) -> Vec<&'a wgpu::TextureView> {
        Vec::new()
    }
}

impl<'a, T: ColorTexel> ColorAttachments<'a, T> for RenderTarget<'a, T> {
    fn views(&self) -> Vec<&'a wgpu::TextureView> {
        vec![self.view()]
    }
}

impl<'a, D: DepthElement> DepthStencilAttachment<'a, Depth<D>> for RenderTarget<'a, Depth<D>>
where
    Depth<D>: Texel + DepthStencilLayout,
{
    fn view(&self) -> Option<&'a wgpu::TextureView> {
        Some(self.view)
    }
}

impl<'a, S: StencilElement> DepthStencilAttachment<'a, Stencil<S>> for RenderTarget<'a, Stencil<S>>
where
    Stencil<S>: Texel + DepthStencilLayout,
{
    fn view(&self) -> Option<&'a wgpu::TextureView> {
        Some(self.view)
    }
}
