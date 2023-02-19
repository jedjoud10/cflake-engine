use crate::{
    ColorTexel, Depth, DepthElement, DepthStencil, LoadOp, Stencil,
    StencilElement, StoreOp, Texel, TexelInfo, Texture, Texture2D,
};

// An attachment layout is a tuple that contains multiple color texels
pub trait ColorLayout {
    // Get the untyped texel info for this layout
    fn layout_info() -> Vec<TexelInfo>;
}

// Singular color attachment
impl<T: ColorTexel> ColorLayout for T {
    fn layout_info() -> Vec<TexelInfo> {
        vec![T::info()]
    }
}

// An attachment layout that contains a depth and/or a stencil texel
pub trait DepthStencilLayout {
    // Get the texel info of the depth stencil texture
    fn info() -> Option<TexelInfo>;

    // Does the DepthStencilLayout contain a Depth format?
    fn is_depth_enabled() -> bool;

    // Does the DepthStencilLayout contain a Stencil format?
    fn is_stencil_enabled() -> bool;
}

// Null depth/stencil attachment, meaning we must disable depth/stencil
impl DepthStencilLayout for () {
    fn info() -> Option<TexelInfo> {
        None
    }

    fn is_depth_enabled() -> bool {
        false
    }

    fn is_stencil_enabled() -> bool {
        false
    }
}

// Depth only attachment
impl<D: DepthElement> DepthStencilLayout for Depth<D>
where
    Self: Texel,
{
    fn info() -> Option<TexelInfo> {
        Some(<Self as Texel>::info())
    }

    fn is_depth_enabled() -> bool {
        true
    }

    fn is_stencil_enabled() -> bool {
        false
    }
}

// Stencil only attachment
impl<S: StencilElement> DepthStencilLayout for Stencil<S>
where
    Self: Texel,
{
    fn info() -> Option<TexelInfo> {
        Some(<Self as Texel>::info())
    }

    fn is_depth_enabled() -> bool {
        false
    }

    fn is_stencil_enabled() -> bool {
        true
    }
}
