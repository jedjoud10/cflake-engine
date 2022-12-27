use std::marker::PhantomData;
use crate::{AnyElement, Normalized};

// The channels that represent the texels (non sRGB)
pub struct R<T: AnyElement>(PhantomData<T>);
pub struct RG<T: AnyElement>(PhantomData<vek::Vec2<T>>);
pub struct RGB<T: AnyElement>(PhantomData<vek::Vec3<T>>);
pub struct RGBA<T: AnyElement>(PhantomData<vek::Vec4<T>>);

// TODO: Implement SRGB

// Element used only for depth texels
pub trait DepthElement: AnyElement {}
impl DepthElement for Normalized<u16> {}
impl DepthElement for Normalized<f32> {}
impl DepthElement for f32 {}

// Element used only for stencil texels
pub trait StencilElement: AnyElement {}
impl StencilElement for u8 {}

// Unique depth and stencil channels for depth render textures and stencil render textures
pub struct Depth<T: DepthElement>(PhantomData<T>);
pub struct Stencil<T: StencilElement>(PhantomData<T>);

// Color channel as texel channels
pub enum ColorChannels {
    R, RG, RGB, RGBA,
}

// Untyped representation of texel channels
pub enum ChannelsType {
    Color(ColorChannels),
    Depth,
    Stencil
}

impl ChannelsType {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            ChannelsType::Color(color) => match color {
                ColorChannels::R => 1,
                ColorChannels::RG => 2,
                ColorChannels::RGB => 3,
                ColorChannels::RGBA => 4,
            },
            ChannelsType::Depth | ChannelsType::Stencil => 1,
        }
    }
}