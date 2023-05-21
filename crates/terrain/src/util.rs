use graphics::{
    Buffer, BufferMode, BufferUsage, Graphics, SamplerSettings, Texel, Texture, Texture3D,
    TextureMipMaps, TextureMode, TextureUsage, TriangleBuffer, Vertex, XYZW, XY, GpuPod, DrawIndexedIndirect,
};
use math::{Node, Octree};
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

// Gets the direction in which we must generate the skirts
// Bit 1 = Start X
// Bit 2 = Start Y
// Bit 3 = Start Z
// Bit 4 = End X
// Bit 5 = End Y
// Bit 6 = End Z
pub(crate) fn find_skirts_direction(node: Node, octree: &Octree) -> u32 {
    let mut skirts = 0u32;
    let nodes = octree.nodes();

    /*
    let mut current = &node;
    for direction in 0..6u32 {
        // Check if the node is in the proper direction relative to parent
        loop {
            let index = node.index();
            let parent = node.parent().unwrap();
            let sibling_base = nodes[parent].children().unwrap().get(); 
            let local_index_relative_to_parent = index - sibling_base;
            let offset = math::CHILDREN_OFFSETS[local_index_relative_to_parent];
        }

    }
    */

    skirts
}