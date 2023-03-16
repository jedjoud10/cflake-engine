use crate::{
    AnyElement, ColorTexel, Depth, DepthElement, ElementType, GpuPod,
    Normalized, Stencil, TexelChannels, VertexChannels, BGRA, R, RG,
    RGBA, SBGRA, SRGBA, UBC1, UBC2, UBC3, UBC7, UBC4, SBC4, SBC5, UBC5,
};
use half::f16;
use std::{any::Any, mem::size_of, ops::Add};
use vek::{
    num_traits::{NumAssignOps, NumOps},
    Vec2, Vec3, Vec4,
};
use wgpu::TextureFormat;

// This trait defines the layout for a single texel that will be stored within textures
// The texel format of each texture is specified at compile time
pub trait Texel: 'static {
    // The raw RAW data type (u8 or shit like dat)
    type Base: GpuPod;

    // The raw vector data type that we will use to access texture memory
    type Storage: GpuPod
        + NumOps<Self::Storage>
        + NumAssignOps<Self::Storage>;

    // Number of bytes in total
    fn size() -> u32 {
        Self::bytes_per_channel() * Self::channels().count()
    }

    // Number of bytes per channel
    // Note: This does not take in consideration the "actual" sizes of each channel (compressed formats)
    fn bytes_per_channel() -> u32;

    // Number of bytes per texel (stride)
    fn stride() -> u32 {
        Self::bytes_per_channel() * Self::channels().count()
    }

    // Untyped representation of the underlying element
    fn element() -> ElementType;

    // Type of channels (either R, RG, RGBA, BGRA, Depth, Stencil)
    fn channels() -> TexelChannels;

    // Compile time WGPU format
    fn format() -> TextureFormat;

    // Get the untyped texel info
    fn info() -> TexelInfo {
        TexelInfo {
            bytes_per_channel: Self::bytes_per_channel(),
            element: Self::element(),
            channels: Self::channels(),
            format: Self::format(),
        }
    }
}

// Untyped texel info that does not contain typed information about the texel nor base types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TexelInfo {
    bytes_per_channel: u32,
    element: ElementType,
    channels: TexelChannels,
    format: TextureFormat,
}

impl TexelInfo {
    // Number of bytes in total
    pub fn size(&self) -> u32 {
        self.bytes_per_channel * self.channels.count()
    }

    // Number of bytes per channel
    pub fn bytes_per_channel(&self) -> u32 {
        self.bytes_per_channel
    }

    // Untyped representation of the underlying element
    pub fn element(&self) -> ElementType {
        self.element
    }

    // Type of channels (either R, RG, RGBA, BGRA, Depth, Stencil)
    pub fn channels(&self) -> TexelChannels {
        self.channels
    }

    // Compile time WGPU format
    pub fn format(&self) -> TextureFormat {
        self.format
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

            fn channels() -> TexelChannels {
                $channels
            }

            fn format() -> TextureFormat {
                // TODO: Check if this gets resolved at compile time?
                crate::pick_texture_format(
                    Self::element(),
                    Self::channels(),
                )
                .unwrap()
            }
        }
    };
}

macro_rules! impl_color_texels {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_texel!($vec, u8, $channels, $storagevec);
        internal_impl_texel!($vec, i8, $channels, $storagevec);
        internal_impl_texel!(
            $vec,
            Normalized<u8>,
            $channels,
            $storagevec
        );
        internal_impl_texel!(
            $vec,
            Normalized<i8>,
            $channels,
            $storagevec
        );

        internal_impl_texel!($vec, u16, $channels, $storagevec);
        internal_impl_texel!($vec, i16, $channels, $storagevec);
        internal_impl_texel!(
            $vec,
            Normalized<u16>,
            $channels,
            $storagevec
        );
        internal_impl_texel!(
            $vec,
            Normalized<i16>,
            $channels,
            $storagevec
        );

        internal_impl_texel!($vec, u32, $channels, $storagevec);
        internal_impl_texel!($vec, i32, $channels, $storagevec);

        internal_impl_texel!($vec, f16, $channels, $storagevec);
        internal_impl_texel!($vec, f32, $channels, $storagevec);
    };
}

macro_rules! impl_compressed_rgba_variants {
    ($elem:ty) => {
        internal_impl_texel!(RGBA, $elem, TexelChannels::Four { swizzled: false }, Vec4);
        internal_impl_texel!(SRGBA, $elem, TexelChannels::Srgba { swizzled: false }, Vec4);
    };
}

macro_rules! impl_compressed_signed_unsigned_variants {
    ($vec:ident, $channels:expr, $storagevec: ident, $unsigned:ty, $signed:ty) => {
        internal_impl_texel!($vec, $unsigned, $channels, $storagevec);
        internal_impl_texel!($vec, $signed, $channels, $storagevec);
    };
}

// Implement basic formats 
type Scalar<T> = T;
impl_color_texels!(
    R,
    TexelChannels::One,
    Scalar
);
impl_color_texels!(
    RG,
    TexelChannels::Two,
    Vec2
);
impl_color_texels!(
    RGBA,
    TexelChannels::Four { swizzled: false },
    Vec4
);
internal_impl_texel!(
    BGRA,
    Normalized<u8>,
    TexelChannels::Four { swizzled: true },
    Vec4
);

// Implement basic SRGBA formats
internal_impl_texel!(
    SRGBA,
    Normalized<u8>,
    TexelChannels::Srgba { swizzled: false },
    Vec4
);
internal_impl_texel!(
    SBGRA,
    Normalized<u8>,
    TexelChannels::Srgba { swizzled: true },
    Vec4
);

// RGBA<Normalized<UBC1>> R5G6B5A1
// SRGBA<Normalized<UBC1>> R5G6B5A1
impl_compressed_rgba_variants!(
    Normalized<UBC1>
);

// RGBA<Normalized<UBC2>> R5G6B5A4
// SRGBA<Normalized<UBC2>> R5G6B5A4
impl_compressed_rgba_variants!(
    Normalized<UBC2>
);

// RGBA<Normalized<UBC2>> R5G6B5A8
// SRGBA<Normalized<UBC2>> R5G6B5A8
impl_compressed_rgba_variants!(
    Normalized<UBC3>
);

// R<Normalized<UBC4>> 
// R<Normalized<SBC4>>
impl_compressed_signed_unsigned_variants!(
    R,
    TexelChannels::Two,
    Vec2,
    Normalized<UBC4>,
    Normalized<SBC4>
);

// RG<Normalized<UBC5>>
// RG<Normalized<SBC5>>
impl_compressed_signed_unsigned_variants!(
    R,
    TexelChannels::Two,
    Vec2,
    Normalized<UBC5>,
    Normalized<SBC5>
);

// RGBA<Normalized<UBC7>>
// SRGBA<Normalized<UBC7>>
impl_compressed_rgba_variants!(
    Normalized<UBC7>
);

// Implement special depth / stencil formats
internal_impl_texel!(
    Depth,
    Normalized<u16>,
    TexelChannels::Depth,
    Scalar
);
internal_impl_texel!(Depth, f32, TexelChannels::Depth, Scalar);
internal_impl_texel!(Stencil, u8, TexelChannels::Stencil, Scalar);
