use vulkan::vk;
use crate::{LoadOp, Stencil, StoreOp, Texel, UntypedTexel, Texture2D, ColorTexel, Texture, UntypedLoadOp, DepthElement, StencilElement, Depth, DepthStencil};

// Describes what an attachment should do within a render pass
// This doesn't store the actual render pass, just a description of it 
pub struct AttachmentDescriptior<T: Texel> {
    pub load_op: LoadOp<T>,
    pub store_op: StoreOp,
    pub stencil_load_op: LoadOp<Stencil<u8>>,
    pub stencil_store_op: StoreOp,
}

// Untyped attachments description don't contain the texel type 
pub struct UntypedAttachmentDescription {
    format: vk::Format,
    load_op: UntypedLoadOp,
    store_op: StoreOp,
    stencil_load_op: LoadOp<Stencil<u8>>,
    stencil_store_op: StoreOp,
}

// An attachment layout is a tuple that contains multiple color texels
pub trait ColorLayout {
    // Get the underlying untyped color texels
    fn untyped_texels() -> Vec<UntypedTexel>;
}

impl<T: ColorTexel> ColorLayout for T {
    fn untyped_texels() -> Vec<UntypedTexel> {
        vec![T::untyped()]
    }
}

// An attachment layout that contains a depth and/or a stencil texel
pub trait DepthStencilLayout {
    // Try to get the underlying untyped texel
    fn untyped_texel() -> Option<UntypedTexel>;
}

impl DepthStencilLayout for () {
    fn untyped_texel() -> Option<UntypedTexel> {
        None
    }
}

impl<D: DepthElement> DepthStencilLayout for Depth<D> {
    fn untyped_texel() -> Option<UntypedTexel> {
        Some(<Depth::<D> as Texel>::untyped())
    }
}

impl<S: StencilElement> DepthStencilLayout for Stencil<S> {
    fn untyped_texel() -> Option<UntypedTexel> {
        Some(<Stencil::<S> as Texel>::untyped())
    }
}

/*
impl<D: DepthElement, S: StencilElement> DepthStencilLayout for DepthStencil<D, S> {
    fn untyped_texel() -> UntypedTexel {
        <DepthStencil::<D, S> as Texel>::untyped()
    }
}
*/
