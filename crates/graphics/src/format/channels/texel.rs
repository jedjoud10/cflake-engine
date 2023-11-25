use crate::format::{AnyElement, Normalized};
use crate::pod::GpuPod;

// The channels that represent the texels (non sRGB)
pub struct R<T: AnyElement>(T);
pub struct RG<T: AnyElement>(vek::Vec2<T>);
pub struct RGBA<T: AnyElement>(vek::Vec4<T>);

// In WGPU, BGRA supports u8 SNORM only
pub trait Swizzable {}
impl Swizzable for Normalized<u8> {}
pub struct BGRA<T: AnyElement + Swizzable>(vek::Vec4<T>);

// In WGPU, SRGBA is only supported by Normalized<u8> and compressed formats
pub trait SupportsSrgba {}
impl SupportsSrgba for Normalized<u8> {}
pub struct SRGBA<T: AnyElement + SupportsSrgba>(vek::Vec4<T>);
pub struct SBGRA<T: AnyElement + Swizzable + SupportsSrgba>(vek::Vec4<T>);

// Element used only for depth-only texels
pub trait DepthElement: AnyElement {}
impl DepthElement for Normalized<u16> {}
impl DepthElement for f32 {}

// Element used for stencil-only texels
pub trait StencilElement: AnyElement {}
impl StencilElement for u8 {}

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: DepthElement>(T);
pub struct Stencil<T: StencilElement>(T);
pub struct DepthStencil<D: DepthElement, S: StencilElement>(D, S);
