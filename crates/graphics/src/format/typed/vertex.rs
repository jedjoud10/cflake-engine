use wgpu::VertexFormat;

use crate::{VectorChannels, ElementType, GpuPodRelaxed};

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