use crate::{
    object::{ToGlName, ToGlTarget},
    prelude::{ColorTexel, DepthTexel, StencilTexel, Texel, Texture, Texture2D},
};

// The canvas layout will be implemented for canvas attachment tuples
pub trait CanvasLayout {
    fn resize(&mut self, size: vek::Extent2<u16>);
}

// Color canvas attachments are texture2D that use the coloe texel types
pub trait ColorCanvasAttachment: Texture {}
impl<T: ColorTexel> ColorCanvasAttachment for Texture2D<T> {}
