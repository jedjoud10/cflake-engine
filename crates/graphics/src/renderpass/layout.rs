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

// An attachment layout is a tuple that contains multiple color attachments
pub trait ColorLayout {
}

// An attachment layout that contains a depth and/or a stencil attachment
pub trait DepthStencilLayout {
}

impl<D: DepthElement> DepthStencilLayout for Depth<D> {}
impl<S: StencilElement> DepthStencilLayout for Stencil<S> {}
impl<D: DepthElement, S: StencilElement> DepthStencilLayout for DepthStencil<D, S> {}
