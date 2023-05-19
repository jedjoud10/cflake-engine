use graphics::{
    Buffer, BufferMode, BufferUsage, Graphics, SamplerSettings, Texel, Texture, Texture3D,
    TextureMipMaps, TextureMode, TextureUsage, TriangleBuffer, Vertex, XYZW, XY, GpuPod, DrawIndexedIndirect,
};
use rendering::{attributes, AttributeBuffer};

// Common types used througohut the crate
pub type TempVertices = Buffer<<XY<f32> as Vertex>::Storage>;
pub type TempTriangles = Buffer<[u32; 3]>;
pub type Vertices = AttributeBuffer<attributes::Position>;
pub type Triangles = TriangleBuffer<u32>;

// Default value for the indexed indirect
pub(crate) const DEFAULT_DRAW_INDEXED_INDIRECT: DrawIndexedIndirect = DrawIndexedIndirect {
    vertex_count: 0,
    instance_count: 1,
    base_index: 0,
    vertex_offset: 0,
    base_instance: 0,
};

// Create counters that will help us generate the vertices
pub(crate) fn create_counters(graphics: &Graphics, count: usize, extra: BufferUsage) -> Buffer<u32> {
    Buffer::zeroed(
        graphics,
        count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | extra,
    )
    .unwrap()
}

// Create an empty buffer that can be written to, copied from/to, and used as storage
pub(crate) fn create_empty_buffer<T: GpuPod, const TYPE: u32>(graphics: &Graphics) -> Buffer<T, TYPE> {
    Buffer::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::COPY_SRC | BufferUsage::COPY_DST | BufferUsage::WRITE | BufferUsage::STORAGE
    ).unwrap()
}

// Create a 3D storage texture with null contents with the specified size
pub(crate) fn create_texture3d<T: Texel>(graphics: &Graphics, size: u32) -> Texture3D<T> {
    Texture3D::<T>::from_texels(
        graphics,
        None,
        vek::Extent3::broadcast(size),
        TextureMode::Dynamic,
        TextureUsage::STORAGE | TextureUsage::WRITE,
        None,
        TextureMipMaps::Disabled,
    )
    .unwrap()
}
