use crate::{AnyElement, GpuPodRelaxed, Normalized};

// The channels that represent the texels (non sRGB)
pub struct R<T: AnyElement>(T);
pub struct RG<T: AnyElement>(vek::Vec2<T>);
pub struct RGBA<T: AnyElement>(vek::Vec4<T>);

// In WGPU, BGRA supports u8 SNORM only
pub trait Swizzable {}
impl Swizzable for Normalized<u8> {}
pub struct BGRA<T: AnyElement + Swizzable>(vek::Vec3<T>);

// The channels that represent the vertices
pub struct X<T: AnyElement>(T);
pub struct XY<T: AnyElement>(vek::Vec2<T>);
pub struct XYZ<T: AnyElement>(vek::Vec3<T>);
pub struct XYZW<T: AnyElement>(vek::Vec4<T>);

// TODO: Implement SRGB

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

// Vector channel as texel channels
// TODO: Is there a better way to handle this?
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VectorChannels {
    One,           // X or R
    Two,           // XY or RG
    Three,         // XYZ or RGB
    ThreeSwizzled, // ZYX or BGR
    Four,          // XYZW or RGBA
    FourSwizzled,  // ZYXW or BGRA
}

impl VectorChannels {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            VectorChannels::One => 1,
            VectorChannels::Two => 2,
            VectorChannels::Three => 3,
            VectorChannels::Four => 4,
            VectorChannels::ThreeSwizzled => 3,
            VectorChannels::FourSwizzled => 4,
        }
    }

    // Check if the R (X) and B (Z) channels are swizzled
    pub const fn is_swizzled(&self) -> bool {
        match self {
            VectorChannels::ThreeSwizzled
            | VectorChannels::FourSwizzled => true,
            _ => false,
        }
    }
}

// Untyped representation of texel channels
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChannelsType {
    Vector(VectorChannels),
    Depth,
    Stencil,
}

impl ChannelsType {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            Self::Vector(color) => color.count(),
            Self::Depth | Self::Stencil => 1,
        }
    }
}
