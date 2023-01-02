use crate::{AnyElement, Normalized};
use std::marker::PhantomData;

// The channels that represent the texels (non sRGB)
pub struct R<T: AnyElement>(T);
pub struct RG<T: AnyElement>(vek::Vec2<T>);
pub struct RGB<T: AnyElement>(vek::Vec3<T>);
pub struct RGBA<T: AnyElement>(vek::Vec4<T>);

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

// TODO: Implement depth-stencil texels

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: DepthElement>(T);
pub struct Stencil<T: StencilElement>(T);

// Vector channel as texel channels
// TODO: Is there a better way to handle this?
pub enum VectorChannels {
    One, // X or R
    Two, // XY or RG
    Three, // XYZ or RGB
    Four, // XYZW or RGBA
}

impl VectorChannels {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            VectorChannels::One => 1,
            VectorChannels::Two => 2,
            VectorChannels::Three => 3,
            VectorChannels::Four => 4,
        }
    }
}

// Untyped representation of texel channels
pub enum ChannelsType {
    Vector(VectorChannels),
    Depth,
    Stencil,
}

impl ChannelsType {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            ChannelsType::Vector(color) => color.count(),
            ChannelsType::Depth | ChannelsType::Stencil => 1,
        }
    }
}
