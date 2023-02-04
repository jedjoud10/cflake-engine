use crate::{
    AnyElement, ChannelsType, ElementType, GpuPodRelaxed,
    VectorChannels, X, XY, XYZ, XYZW,
};
use std::mem::size_of;
use vek::{Vec2, Vec3, Vec4};
use wgpu::VertexFormat;

// A vertex that represents a vertex within a rendered object
pub trait Vertex {
    // The raw data type that we will use to access texture memory
    type Storage: GpuPodRelaxed;

    // Number of bits per axii
    fn bits_per_channel() -> u64;

    // Untyped representation of the underlying element
    fn element() -> ElementType;

    // Type of channels (either X, XY, XYZ, XYZW)
    fn channels() -> VectorChannels;

    // Compile time WGPU format
    fn format() -> VertexFormat;
}

// Equivalent to vk::VertexInputAttributeDescription
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    //pub format: UntypedVertex,
    pub binding: u32,
    pub location: u32,
    pub offset: u32,
}

// Equivalent to vk::VertexInputBindingDescription
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexBinding {
    //pub format: UntypedVertex,
    pub binding: u32,
}

// Grapics pipeline vertex configuration
pub struct VertexConfig {
    pub attributes: Vec<VertexAttribute>,
    pub bindings: Vec<VertexBinding>,
}

// Implement the vertex texel layout
macro_rules! impl_vector_texel_layout {
    ($t:ident, $channels_type:expr, $vec: ident) => {
        impl<T: AnyElement> Vertex for $t<T> {
            type Storage = $vec<T::Storage>;

            fn bits_per_channel() -> u64 {
                size_of::<T>() as u64 * 8
            }

            fn element() -> ElementType {
                T::ELEMENT_TYPE
            }

            fn channels() -> VectorChannels {
                $channels_type
            }

            fn format() -> wgpu::VertexFormat {
                todo!()
            }
        }
    };
}

// Need this for the macro to work
type Scalar<T> = T;

impl_vector_texel_layout!(X, VectorChannels::One, Scalar);
impl_vector_texel_layout!(XY, VectorChannels::Two, Vec2);
impl_vector_texel_layout!(XYZ, VectorChannels::Three, Vec3);
impl_vector_texel_layout!(XYZW, VectorChannels::Four, Vec4);
