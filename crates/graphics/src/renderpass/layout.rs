use crate::{
    ColorTexel, Depth, DepthElement, DepthStencil, LoadOp, Stencil,
    StencilElement, StoreOp, Texel, Texture, Texture2D,
};

// Describes what an attachment should do within a render pass
// This doesn't store the actual render pass, just a description of it
pub struct AttachmentDescriptior<T: Texel> {
    pub load_op: LoadOp<T>,
    pub store_op: StoreOp,
}

// An attachment layout is a tuple that contains multiple color texels
pub trait ColorLayout {
}

// Singular color attachment
impl<T: ColorTexel> ColorLayout for T {
}

// An attachment layout that contains a depth and/or a stencil texel
pub trait DepthStencilLayout {
}

// Null depth/stencil attachment, meaning we must disable depth/stencil
impl DepthStencilLayout for () {
}

// Depth only attachment
impl<D: DepthElement> DepthStencilLayout for Depth<D> {
}

// Stencil only attachment
impl<S: StencilElement> DepthStencilLayout for Stencil<S> {
}

/*
impl<D: DepthElement, S: StencilElement> DepthStencilLayout for DepthStencil<D, S> {
    fn untyped_texel() -> UntypedTexel {
        <DepthStencil::<D, S> as Texel>::untyped()
    }
}
*/
