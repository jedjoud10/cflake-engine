use super::AnyElement;
use crate::{
    Base, BaseType, ChannelsType, ColorChannels, Depth, DepthElement,
    ElementType, Normalizable, Normalized, Stencil, StencilElement,
    R, RG, RGB, RGBA,
};
use std::mem::size_of;
use vek::{Vec2, Vec3, Vec4};
use vulkan::vk;

// An untyped wrapper around texel types
pub struct UntypedTexel {
    // Format related
    pub format: vk::Format,
    pub channels: ChannelsType,
    pub element: ElementType,

    // Storage/memory related
    pub total_bits: u32,
    pub bits_per_channel: u32,
}

// This trait defines the layout for a single texel that will be stored within textures
// The texel format of each texture is specified at compile time
// This assumes a very simple case of multi-channel texels
pub trait Texel: 'static + Sized {
    // Number of bits per channel
    const BITS_PER_CHANNEL: u32;

    // Untyped representation of the underlying element
    const ELEMENT_TYPE: ElementType;

    // Type of channels (either R, RG, RGB, RGBA, Depth, Stencil)
    const CHANNELS_TYPE: ChannelsType;

    // Compile time Vulkan format (calls to cases::guess)
    const FORMAT: vk::Format;

    // The raw data type that we will use to access texture memory
    type Storage;

    // Get the untyped variant of this texel
    fn untyped() -> UntypedTexel {
        UntypedTexel {
            format: Self::FORMAT,
            channels: Self::CHANNELS_TYPE,
            element: Self::ELEMENT_TYPE,
            bits_per_channel: Self::BITS_PER_CHANNEL,
            total_bits: Self::BITS_PER_CHANNEL
                * Self::CHANNELS_TYPE.count(),
        }
    }
}

// Implement the color texel layout
macro_rules! impl_color_texel_layout {
    ($t:ident, $channels_type:expr, $vec: ident) => {
        impl<T: AnyElement> Texel for $t<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = $channels_type;
            const FORMAT: vk::Format = super::pick_format_from_params(
                Self::ELEMENT_TYPE,
                Self::CHANNELS_TYPE,
            );
            type Storage = $vec<T>;
        }
    };
}

// Implement the special texel layouts
macro_rules! impl_special_texel_layout {
    () => {
        impl<T: DepthElement> Texel for Depth<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = ChannelsType::Depth;
            const FORMAT: vk::Format = super::pick_format_from_params(
                Self::ELEMENT_TYPE,
                Self::CHANNELS_TYPE,
            );
            type Storage = T;
        }

        impl<T: StencilElement> Texel for Stencil<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = ChannelsType::Stencil;
            const FORMAT: vk::Format = super::pick_format_from_params(
                Self::ELEMENT_TYPE,
                Self::CHANNELS_TYPE,
            );
            type Storage = T;
        }
    };
}

// Need this for the macro to work
type Scalar<T> = T;

impl_color_texel_layout!(
    R,
    ChannelsType::Color(ColorChannels::R),
    Scalar
);
impl_color_texel_layout!(
    RG,
    ChannelsType::Color(ColorChannels::RG),
    Vec2
);
impl_color_texel_layout!(
    RGB,
    ChannelsType::Color(ColorChannels::RGB),
    Vec3
);
impl_color_texel_layout!(
    RGBA,
    ChannelsType::Color(ColorChannels::RGBA),
    Vec4
);
impl_special_texel_layout!();
