use crate::{
    AnyElement, ElementType, VectorChannels, X, XY, XYZ, XYZW, GpuPodRelaxed,
};
use std::{mem::size_of};
use vek::{Vec2, Vec3, Vec4};
use vulkan::vk;

// An untyped wrapper around vertex types
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UntypedVertex {
    // Format related
    pub format: vk::Format,
    pub channels: VectorChannels,
    pub element: ElementType,

    // Storage/memory related
    pub bits_per_axii: u64,
}

// A vertex that represents a vertex within a rendered object
pub trait Vertex {
    // Number of bits per axii
    const BITS_PER_AXII: u64;

    // Untyped representation of the underlying element
    const ELEMENT_TYPE: ElementType;

    // Type of vector channels (either X, XY, XYZ, XYZW)
    const VECTOR_CHANNELS_TYPE: VectorChannels;

    // Compile time Vulkan format (calls to cases::guess)
    const FORMAT: vk::Format;

    // The raw data type that we will use to access texture memory
    type Storage: GpuPodRelaxed;

    // Get the untyped variant of this texel
    fn untyped() -> UntypedVertex {
        UntypedVertex {
            format: Self::FORMAT,
            channels: Self::VECTOR_CHANNELS_TYPE,
            element: Self::ELEMENT_TYPE,
            bits_per_axii: Self::BITS_PER_AXII,
        }
    }
}

// Equivalent to vk::VertexInputAttributeDescription
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    pub format: UntypedVertex,
    pub binding: u32,
    pub location: u32,
    pub offset: u32,
}

// Equivalent to vk::VertexInputBindingDescription
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexBinding {
    pub format: UntypedVertex,
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
            const BITS_PER_AXII: u64 = size_of::<T>() as u64 * 8;
            const ELEMENT_TYPE: ElementType = T::ELEMENT_TYPE;
            const VECTOR_CHANNELS_TYPE: VectorChannels =
                $channels_type;
            const FORMAT: vk::Format =
                crate::format::pick_format_from_vector_channels(
                    Self::ELEMENT_TYPE,
                    Self::VECTOR_CHANNELS_TYPE,
                );
            type Storage = $vec<T::Storage>;
        }
    };
}


// Need this for the macro to work
type Scalar<T> = T;

impl_vector_texel_layout!(X, VectorChannels::One, Scalar);
impl_vector_texel_layout!(XY, VectorChannels::Two, Vec2);
impl_vector_texel_layout!(XYZ, VectorChannels::Three, Vec3);
impl_vector_texel_layout!(XYZW, VectorChannels::Four, Vec4);
