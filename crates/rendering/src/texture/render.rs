use world::{Handle, UntypedHandle, Storage};
use crate::{canvas::Canvas, object::ToGlName};
use super::{Texture2D, Texel, TexelFormat, Texture, TextureMode};

// A render texture is a texture that will be used within a canvas
// The render texture trait is implement for Texture2D with color and depth/stencil layouts
// Handle trait implemented for all (Handle<T>, Storage<T>) of RenderTextures

// TODO: Fix comments
pub trait RenderTexture {
    fn texel_format(&self) -> TexelFormat;
    fn size(&self) -> vek::Extent2<u16>;
    fn mode(&self) -> TextureMode;
    fn name(&self) -> u32;
}

impl<T: Texel> RenderTexture for Texture2D<T> {
    fn texel_format(&self) -> TexelFormat {
        T::ENUM_FORMAT
    }

    fn name(&self) -> u32 {
        <Self as ToGlName>::name(self)
    }

    fn size(&self) -> vek::Extent2<u16> {
        self.region().1
    }

    fn mode(&self) -> TextureMode {
        <Self as Texture>::mode(self)
    }
}