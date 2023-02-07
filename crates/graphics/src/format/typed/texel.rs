use vek::{Vec3, Vec2, Vec4};
use wgpu::TextureFormat;
use half::f16;
use std::mem::size_of;
use crate::{GpuPodRelaxed, ElementType, ChannelsType, VectorChannels, R, RG, RGBA, AnyElement, BGRA, Normalized, DepthElement, Depth, Stencil, TexelInfo};

// This trait defines the layout for a single texel that will be stored within textures
// The texel format of each texture is specified at compile time
pub trait Texel: 'static + Sized {
    // The raw RAW data type (u8 or shit like dat)
    type Base: GpuPodRelaxed;

    // The raw vector data type that we will use to access texture memory
    type Storage: GpuPodRelaxed;

    // Number of bytes in total
    fn size() -> u32 {
        Self::bytes_per_channel() * Self::channels().count()
    }

    // Number of bytes per channel
    fn bytes_per_channel() -> u32;

    // Untyped representation of the underlying element
    fn element() -> ElementType;

    // Type of channels (either R, RG, RGBA, BGRA, Depth, Stencil)
    fn channels() -> ChannelsType;

    // Compile time WGPU format
    fn format() -> TextureFormat;

    // Get the untyped texel info
    fn info() -> TexelInfo {
        TexelInfo {
            bytes_per_channel: Self::bytes_per_channel(),
            element: Self::element(),
            channels: Self::channels(),
            format: Self::format()
        }
    }
}

macro_rules! internal_impl_texel {
    ($vec:ident, $elem:ty, $channels:expr, $storagevec: ident) => {
        impl Texel for $vec<$elem> {
            type Base = <$elem as AnyElement>::Storage;
            type Storage = $storagevec<Self::Base>;

            fn bytes_per_channel() -> u32 {
                size_of::<$elem>() as u32
            }

            fn element() -> ElementType {
                <$elem as AnyElement>::ELEMENT_TYPE
            }

            fn channels() -> ChannelsType {
                $channels
            }

            fn format() -> TextureFormat {
                // TODO: Check if this gets resolved at compile time?
                crate::pick_texture_format(
                    Self::element(),
                    Self::channels(),
                ).unwrap()
            }
        }
    };
}

macro_rules! impl_color_texels {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_texel!($vec, u8, $channels, $storagevec);
        internal_impl_texel!($vec, i8, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<u8>, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<i8>, $channels, $storagevec);

        internal_impl_texel!($vec, u16, $channels, $storagevec);
        internal_impl_texel!($vec, i16, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<u16>, $channels, $storagevec);
        internal_impl_texel!($vec, Normalized<i16>, $channels, $storagevec);

        internal_impl_texel!($vec, u32, $channels, $storagevec);
        internal_impl_texel!($vec, i32, $channels, $storagevec);

        internal_impl_texel!($vec, f16, $channels, $storagevec);
        internal_impl_texel!($vec, f32, $channels, $storagevec);
    };  
}

type Scalar<T> = T;
impl_color_texels!(R, ChannelsType::Vector(VectorChannels::One), Scalar);
impl_color_texels!(RG, ChannelsType::Vector(VectorChannels::Two), Vec2);
impl_color_texels!(RGBA, ChannelsType::Vector(VectorChannels::Four), Vec4);
internal_impl_texel!(BGRA, Normalized<u8>, ChannelsType::Vector(VectorChannels::FourSwizzled), Vec4);

internal_impl_texel!(Depth, Normalized<u16>, ChannelsType::Depth, Scalar);
internal_impl_texel!(Depth, f32, ChannelsType::Depth, Scalar);
internal_impl_texel!(Stencil, u8, ChannelsType::Stencil, Scalar);