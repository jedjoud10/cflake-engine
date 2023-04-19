use crate::{
    AnyElement, Depth, DepthElement, ElementType, GpuPod, Normalized,
    Stencil, TexelChannels, VertexChannels, X, XY, XYZ, XYZW,
};
use half::f16;
use std::mem::size_of;
use vek::{
    num_traits::{NumAssignOps, NumOps},
    Vec2, Vec3, Vec4,
};
use wgpu::VertexFormat;

// A vertex that represents a vertex within a rendered object
pub trait Vertex {
    // The raw RAW data type (u8 or shit like dat)
    type Base: GpuPod;

    // The raw data type that we will use to access vertex memory
    type Storage: GpuPod
        + NumOps<Self::Storage>
        + NumAssignOps<Self::Storage>;

    // Number of bytes in total
    fn size() -> u32 {
        Self::bytes_per_channel() * Self::channels().count()
    }

    // Number of bytes per channel
    fn bytes_per_channel() -> u32;

    // Untyped representation of the underlying element
    fn element() -> ElementType;

    // Type of channels (either X, XY, XYZ, XYZW)
    fn channels() -> VertexChannels;

    // Compile time WGPU format
    fn format() -> VertexFormat;

    // Get the untyped vertex info
    fn info() -> VertexInfo {
        VertexInfo {
            bytes_per_channel: Self::bytes_per_channel(),
            element: Self::element(),
            channels: Self::channels(),
            format: Self::format(),
        }
    }
}

// Untyped texel info that does not contain typed information about the vertex nor base types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexInfo {
    bytes_per_channel: u32,
    element: ElementType,
    channels: VertexChannels,
    format: VertexFormat,
}

impl VertexInfo {
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

    // Type of channels (either X, XY, XYZ, XYZW)
    pub fn channels(&self) -> VertexChannels {
        self.channels
    }

    // Compile time WGPU format
    pub fn format(&self) -> VertexFormat {
        self.format
    }
}

macro_rules! internal_impl_vertex {
    ($vec:ident, $elem:ty, $channels:expr, $storagevec: ident) => {
        impl Vertex for $vec<$elem> {
            type Base = <$elem as AnyElement>::Storage;
            type Storage = $storagevec<Self::Base>;

            fn bytes_per_channel() -> u32 {
                size_of::<$elem>() as u32
            }

            fn element() -> ElementType {
                <$elem as AnyElement>::ELEMENT_TYPE
            }

            fn channels() -> VertexChannels {
                $channels
            }

            fn format() -> VertexFormat {
                // TODO: Check if this gets resolved at compile time?
                crate::pick_vertex_format(
                    Self::element(),
                    Self::channels(),
                )
                .unwrap()
            }
        }
    };
}

macro_rules! impl_vertex_8 {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_vertex!($vec, u8, $channels, $storagevec);
        internal_impl_vertex!($vec, i8, $channels, $storagevec);
        internal_impl_vertex!(
            $vec,
            Normalized<u8>,
            $channels,
            $storagevec
        );
        internal_impl_vertex!(
            $vec,
            Normalized<i8>,
            $channels,
            $storagevec
        );
    };
}

macro_rules! impl_vertex_16 {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_vertex!($vec, u16, $channels, $storagevec);
        internal_impl_vertex!($vec, i16, $channels, $storagevec);
        internal_impl_vertex!(
            $vec,
            Normalized<u16>,
            $channels,
            $storagevec
        );
        internal_impl_vertex!(
            $vec,
            Normalized<i16>,
            $channels,
            $storagevec
        );
    };
}

macro_rules! impl_vertex_32 {
    ($vec:ident, $channels:expr, $storagevec: ident) => {
        internal_impl_vertex!($vec, u32, $channels, $storagevec);
        internal_impl_vertex!($vec, i32, $channels, $storagevec);
        internal_impl_vertex!($vec, f32, $channels, $storagevec);
    };
}

type Scalar<T> = T;

// 4 bytes for 1 channel (4)
impl_vertex_32!(X, VertexChannels::One, Scalar);

// 2 bytes for 2 channels (4), 4 bytes for 2 channels (8)
impl_vertex_16!(XY, VertexChannels::Two, Vec2);
impl_vertex_32!(XY, VertexChannels::Two, Vec2);

// 1 byte for 2 channels (2)
// FIXME: Cannot have this because of VERTEX_STRIDE_ALIGNMENT 
//impl_vertex_8!(XYZW, VertexChannels::Four, Vec4);

// 4 bytes for 3 channels (12)
impl_vertex_32!(XYZ, VertexChannels::Three, Vec3);

// 1 byte for 4 channels (4),
impl_vertex_8!(XYZW, VertexChannels::Four, Vec4);

// 2 bytes for 4 channels (8),
impl_vertex_16!(XYZW, VertexChannels::Four, Vec4);

// 4 bytes for 4 channels (16),
impl_vertex_32!(XYZW, VertexChannels::Four, Vec4);
