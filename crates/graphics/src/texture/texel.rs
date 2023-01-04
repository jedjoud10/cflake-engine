use crate::{
    AnyElement, ChannelsType, Depth, DepthStencil, DepthElement, ElementType,
    Normalized, Stencil, StencilElement, Swizzable, VectorChannels,
    BGR, BGRA, R, RG, RGB, RGBA, GpuPodRelaxed,
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
    pub bits_per_channel: u64,
}

// This trait defines the layout for a single texel that will be stored within textures
// The texel format of each texture is specified at compile time
// This assumes a very simple case of multi-channel texels
pub trait Texel: 'static + Sized {
    // Number of bits per channel
    const BITS_PER_CHANNEL: u64;

    // Untyped representation of the underlying element
    const ELEMENT_TYPE: ElementType;

    // Type of channels (either R, RG, RGB, RGBA, Depth, Stencil)
    const CHANNELS_TYPE: ChannelsType;

    // Compile time Vulkan format (calls to cases::guess)
    const FORMAT: vk::Format;

    // The raw data type that we will use to access texture memory
    type Storage: GpuPodRelaxed;

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

// Color texels are texels used for color attachments
// TODO: Figure out if there are any limits to this
pub trait ColorTexel: Texel {
    // A texel that represents complete black or 0 (-1 if normalized)
    fn black() -> Self::Storage;

    // A texel that represents gray or 0.5 (0 if normalized)
    fn grey() -> Self::Storage;

    // A texel that represents complete white or 1 (1 if normalized)
    fn white() -> Self::Storage;

    // Convert this texel to a clear color value 
    fn into_clear_color_value(storage: Self::Storage) -> vk::ClearColorValue;
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
            const BITS_PER_CHANNEL: u64 = size_of::<T>() as u64 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = $channels_type;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = $vec<T::Storage>;
        }

        impl<T: AnyElement> ColorTexel for $t<T> {
            fn black() -> Self::Storage {
                todo!()
            }

            fn grey() -> Self::Storage {
                todo!()
            }

            fn white() -> Self::Storage {
                todo!()
            }
            
            fn into_clear_color_value(_storage: Self::Storage) -> vk::ClearColorValue {
                todo!()
            }
        }
    };
}

// Implement the swizzled color texel layout
macro_rules! impl_swizzled_color_texel_layout {
    ($t:ident, $channels_type:expr, $vec: ident) => {
        impl<T: AnyElement + Swizzable> Texel for $t<T> {
            const BITS_PER_CHANNEL: u64 = size_of::<T>() as u64 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = $channels_type;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = $vec<T::Storage>;
        }

        impl<T: AnyElement + Swizzable> ColorTexel for $t<T> {
            fn black() -> Self::Storage {
                todo!()
            }

            fn grey() -> Self::Storage {
                todo!()
            }

            fn white() -> Self::Storage {
                todo!()
            }
            
            fn into_clear_color_value(_storage: Self::Storage) -> vk::ClearColorValue {
                todo!()
            }
        }
    };
}

// Implement the special texel layouts
macro_rules! impl_special_texel_layout {
    () => {
        impl<T: DepthElement> Texel for Depth<T> {
            const BITS_PER_CHANNEL: u64 = size_of::<T>() as u64 * 8;
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
            const BITS_PER_CHANNEL: u64 = size_of::<T>() as u64 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const CHANNELS_TYPE: ChannelsType = ChannelsType::Stencil;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = T::Storage;
        }


        /*
        TODO: Fix this sheize
        impl<D: DepthElement, S: StencilElement> Texel for DepthStencil<D, S> where (D::Storage, S::Storage): GpuPodRelaxed {
            const BITS_PER_CHANNEL: u64 = size_of::<D>() as u64 * 8 + size_of::<S>() as u64 * 8;
            const ELEMENT_TYPE: ElementType = ElementType::CompoundDepthStencil {
                depth_bits: size_of::<D>() as u64 * 8
            };
            const CHANNELS_TYPE: ChannelsType = ChannelsType::Stencil;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_params(
                    Self::ELEMENT_TYPE,
                    Self::CHANNELS_TYPE,
                );
            type Storage = (D::Storage, S::Storage);
        }
        */
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
