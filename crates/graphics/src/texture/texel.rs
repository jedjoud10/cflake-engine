use crate::{
    AnyElement, Base, BaseType, ChannelsType, Depth, DepthElement,
    ElementType, Normalizable, Normalized, Stencil, StencilElement,
    Swizzable, VectorChannels, BGR, BGRA, R, RG, RGB, RGBA,
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
    type Storage: bytemuck::Pod;

    // Get the untyped variant of this texel
    fn untyped() -> UntypedTexel {
        UntypedTexel {
            format: Self::FORMAT,
            channels: Self::CHANNELS_TYPE,
            element: Self::ELEMENT_TYPE,
            bits_per_channel: Self::BITS_PER_CHANNEL,
        }
    }
}

// Image texels are texels that can be loaded from a file, like when loading a Texture2D<RGBA<Normalized<u8>>
pub trait ImageTexel: Texel {
    fn to_image_texels(
        image: image::DynamicImage,
    ) -> Vec<Self::Storage>;
}

// Implement the color texel layout
macro_rules! impl_color_texel_layout {
    ($t:ident, $channels_type:expr, $vec: ident) => {
        impl<T: AnyElement> Texel for $t<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = $channels_type;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = $vec<T::Storage>;
        }
    };
}

// Implement the swizzled color texel layout
macro_rules! impl_swizzled_color_texel_layout {
    ($t:ident, $channels_type:expr, $vec: ident) => {
        impl<T: AnyElement + Swizzable> Texel for $t<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = $channels_type;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = $vec<T::Storage>;
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
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = T::Storage;
        }

        impl<T: StencilElement> Texel for Stencil<T> {
            const BITS_PER_CHANNEL: u32 = size_of::<T>() as u32 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = ChannelsType::Stencil;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = T::Storage;
        }
    };
}

// Internally used for implementing the image texel
macro_rules! internal_impl_single_image_texel {
    ($t:ident, $base:ty, $convert:ident, $closure:expr) => {
        impl ImageTexel for $t<$base> {
            fn to_image_texels(
                image: image::DynamicImage,
            ) -> Vec<Self::Storage> {
                let image = image.$convert();
                image.chunks(4).map($closure).collect()
            }
        }
    };
}

// Implement the image texel layouts
macro_rules! impl_image_texel {
    ($t:ident, $closure:expr) => {
        internal_impl_single_image_texel!(
            $t, u8, into_rgba8, $closure
        );
        internal_impl_single_image_texel!(
            $t,
            u16,
            into_rgba16,
            $closure
        );
        internal_impl_single_image_texel!(
            $t,
            Normalized<u8>,
            into_rgba8,
            $closure
        );
        internal_impl_single_image_texel!(
            $t,
            Normalized<u16>,
            into_rgba16,
            $closure
        );
    };
}

// Need this for the macro to work
type Scalar<T> = T;

impl_color_texel_layout!(
    R,
    ChannelsType::Vector(VectorChannels::One),
    Scalar
);
impl_color_texel_layout!(
    RG,
    ChannelsType::Vector(VectorChannels::Two),
    Vec2
);
impl_color_texel_layout!(
    RGB,
    ChannelsType::Vector(VectorChannels::Three),
    Vec3
);
impl_color_texel_layout!(
    RGBA,
    ChannelsType::Vector(VectorChannels::Four),
    Vec4
);

// Swizzled
impl_swizzled_color_texel_layout!(
    BGR,
    ChannelsType::Vector(VectorChannels::ThreeSwizzled),
    Vec3
);
impl_swizzled_color_texel_layout!(
    BGRA,
    ChannelsType::Vector(VectorChannels::FourSwizzled),
    Vec4
);

// Special
impl_special_texel_layout!();

// Image texels
impl_image_texel!(R, |val| val[0]);
impl_image_texel!(RG, vek::Vec2::from_slice);
impl_image_texel!(RGB, vek::Vec3::from_slice);
impl_image_texel!(RGBA, vek::Vec4::from_slice);
