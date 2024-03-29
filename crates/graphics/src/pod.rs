use std::{
    hash::{Hash, Hasher},
    num::NonZeroU64,
};

use bytemuck::{Pod, Zeroable};
use nohash_hasher::NoHashHasher;

// Plain old data type that can be sent to the gpu
// This is a bit of a hack tbh since bool doesn't implement
pub unsafe trait GpuPod: bytemuck::Pod + bytemuck::Zeroable + Sync + Send + 'static {
    // Convert the data type to raw bytes
    fn into_bytes(&self) -> &[u8] {
        let ptr = self as *const Self;

        // This is safe since the type implements bytemuck::Pod, and we are only casting one element
        unsafe { core::slice::from_raw_parts(ptr as *const u8, Self::size()) }
    }

    // Try converting raw bytes into self
    fn from_bytes(bytes: &[u8]) -> Self {
        let raw: &[Self] = bytemuck::cast_slice(bytes);
        debug_assert_eq!(raw.len(), 1);
        raw[0]
    }

    // Convert a slice of GpuPods into bytes
    fn slice_into_bytes(slice: &[Self]) -> &[u8] {
        bytemuck::cast_slice(slice)
    }

    // Convert a slice of bytes into GpuPods
    fn bytes_into_slice(bytes: &[u8]) -> &[Self] {
        bytemuck::cast_slice(bytes)
    }

    // Get the size of this POD
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    // Get the alignment value of this POD
    fn alignment() -> usize {
        std::mem::align_of::<Self>()
    }

    // Get the untyped GPU pod info
    fn info() -> GpuPodInfo {
        GpuPodInfo {
            size: Self::size(),
            alignment: Self::alignment(),
        }
    }
}

unsafe impl<T: Clone + Copy + Sync + Send + bytemuck::Pod + bytemuck::Zeroable + 'static> GpuPod
    for T
{
}

// Gpu pod info simply contains the size and alignment of a GPU pod type
pub struct GpuPodInfo {
    size: usize,
    alignment: usize,
}

impl GpuPodInfo {
    // Get the size of this POD
    pub fn size(&self) -> usize {
        self.size
    }

    // Get the alignment value of this POD
    pub fn alignment(&self) -> usize {
        self.alignment
    }
}

// Sole reason I copied over the WGPU types is because they don't implement Pod and Zeroable
// Cmon wgpu be better (maybe I should make pull request??)

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct DrawIndirect {
    /// The number of vertices to draw.
    pub vertex_count: u32,
    /// The number of instances to draw.
    pub instance_count: u32,
    /// The Index of the first vertex to draw.
    pub base_vertex: u32,
    /// The instance ID of the first instance to draw.
    /// Has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`](crate::Features::INDIRECT_FIRST_INSTANCE) is enabled.
    pub base_instance: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct DrawIndexedIndirect {
    /// The number of vertices to draw.
    pub vertex_count: u32,
    /// The number of instances to draw.
    pub instance_count: u32,
    /// The base index within the index buffer.
    pub base_index: u32,
    /// The value added to the vertex index before indexing into the vertex buffer.
    pub vertex_offset: i32,
    /// The instance ID of the first instance to draw.
    /// Has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`](crate::Features::INDIRECT_FIRST_INSTANCE) is enabled.
    pub base_instance: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct DispatchIndirect {
    /// The number of work groups in X dimension.
    pub x: u32,
    /// The number of work groups in Y dimension.
    pub y: u32,
    /// The number of work groups in Z dimension.
    pub z: u32,
}

// Unique struct variant for each ID since there might be collisions between IDs of non-equal types

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IdVariant {
    Adapter,
    Device,
    Queue,
    BindGroupLayout,
    BindGroup,
    TextureView,
    Sampler,
    Buffer,
    Texture,
    QuerySet,
    PipelineLayout,
    RenderPipeline,
    ComputePipeline,
    RenderBundle,
    Surface,
}

// A global ID for each object generated by the graphics context
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Id(pub(crate) u64, pub(crate) IdVariant);

struct InternalId(NonZeroU64);
impl nohash_hasher::IsEnabled for InternalId {}
impl Hash for InternalId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.get());
    }
}

impl Id {
    // Create an object ID using a raw wgpu handle and a variant
    pub fn new<T>(raw: wgpu::Id<T>, variant: IdVariant) -> Self {
        let mut hash = NoHashHasher::<InternalId>::default();
        raw.hash(&mut hash);
        let value = hash.finish();
        Self(value, variant)
    }
}
