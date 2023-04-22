use wgpu::BlendState;

use crate::{
    ColorTexel, Depth, DepthElement, DepthStencil, LoadOp, Stencil, StencilElement, StoreOp, Texel,
    TexelInfo, Texture, Texture2D,
};

// An attachment layout is a tuple that contains multiple color texels
pub trait ColorLayout {
    // Array that contains 1-n number of Option<BlendState>
    type BlendingArray: Into<Vec<Option<BlendState>>> + Copy;

    // Get the untyped texel info for this layout
    fn layout_info() -> Vec<TexelInfo>;
}

// No maidens
impl ColorLayout for () {
    type BlendingArray = [Option<BlendState>; 0];

    fn layout_info() -> Vec<TexelInfo> {
        Vec::new()
    }
}

// Singular color attachment
impl<T: ColorTexel> ColorLayout for T {
    type BlendingArray = [Option<BlendState>; 1];

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
