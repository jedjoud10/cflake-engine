use graphics::{
    Buffer, BufferMode, BufferUsage, DrawIndexedIndirect, GpuPod, Graphics, Texel,
    Texture, Texture3D, TextureMipMaps, TextureUsage, TriangleBuffer, Vertex, XY, TextureViewSettings, XYZW,
};
use math::{Node, Octree};
use rendering::{attributes, AttributeBuffer};

// Common types used througohut the crate
pub type TempVertices = Buffer<<XYZW<f32> as Vertex>::Storage>;
pub type TempTriangles = Buffer<[u32; 3]>;
pub type PermVertices = AttributeBuffer<attributes::Position>;
pub type PermTriangles = TriangleBuffer<u32>;

// Default value for the indexed indirect
pub(crate) const DEFAULT_DRAW_INDEXED_INDIRECT: DrawIndexedIndirect = DrawIndexedIndirect {
    vertex_count: 0,
    instance_count: 1,
    base_index: 0,
    vertex_offset: 0,
    base_instance: 0,
};

// Create counters that will help us generate the vertices
pub(crate) fn create_counters(
    graphics: &Graphics,
    count: usize,
    extra: BufferUsage,
) -> Buffer<u32> {
    Buffer::zeroed(
        graphics,
        count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | extra,
    )
    .unwrap()
}

// Create an empty buffer that can be written to, copied from/to, and used as storage
pub(crate) fn create_empty_buffer<T: GpuPod, const TYPE: u32>(
    graphics: &Graphics,
) -> Buffer<T, TYPE> {
    Buffer::from_slice(
        graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::COPY_SRC | BufferUsage::COPY_DST | BufferUsage::WRITE | BufferUsage::STORAGE,
    )
    .unwrap()
}

// Create a 3D storage texture with null contents with the specified size
pub(crate) fn create_texture3d<T: Texel>(graphics: &Graphics, size: u32) -> Texture3D<T> {
    Texture3D::<T>::from_texels(
        graphics,
        None,
        vek::Extent3::broadcast(size),
        TextureUsage::STORAGE | TextureUsage::WRITE,
        &[TextureViewSettings::whole::<<Texture3D<T> as Texture>::Region>()],
        None,
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Generation priority heauristic for chunks
pub(crate) fn generation_priority_heuristic(
    node_center: vek::Vec3<f32>,
    viewer_position: vek::Vec3<f32>,
    viewer_forward: vek::Vec3<f32>,
) -> f32 {
    // Favors chunks that are near the viewer
    let mut priority =
        (1.0 / viewer_position.distance(node_center).max(1.0)) * 10.0;

    // Favors chunks that are in front of the viewer
    priority *= viewer_forward
        .dot((node_center - viewer_position).normalized())
        * 5.0;

    return priority.clamp(0.0f32, 1000.0f32);
}

// Convert packed vertices to normal vertices (with scale) for collisions
pub(crate) fn transform_vertices(
    packed: Vec<vek::Vec4<f32>>,
    node: Node,
) -> Vec<vek::Vec3<f32>> {
    let factor = node.size() as f32 / (node.size() as f32 - 4.0);

    packed.into_iter()
        .map(|float_packed| {
            let uint_packed = float_packed.map(|x| u32::from_ne_bytes(x.to_ne_bytes()));
            let packed_cell_position = uint_packed.x;
            let packed_inner_position = uint_packed.y;

            // Unpack the packed cell position
            let cell_position = {
                let bytes = packed_cell_position.to_ne_bytes();
                vek::Vec4::from_slice(&bytes).map(|byte| byte as f32)
            };
            
            // Unpack the packed inner cell position
            let inner_position = {
                let bytes = packed_inner_position.to_ne_bytes();
                vek::Vec4::from_slice(&bytes).map(|byte| byte as f32 / 128.0f32 - 1.0)
            };

            (cell_position + inner_position).xyz() * factor + vek::Vec3::one()
        })
        .collect()
}

// Gets the direction in which we must generate the skirts
// Bit 1 = Start X
// Bit 2 = Start Y
// Bit 3 = Start Z
// Bit 4 = End X
// Bit 5 = End Y
// Bit 6 = End Z
// TODO: URGENT PLEASE FIX
pub(crate) fn find_skirts_direction(_node: &Node, _octree: &Octree) -> u32 {
    return 0;
    //return u32::MAX;
    /*
    let mut skirts = 0u32;
    let nodes = octree.nodes();

    let mut current = &node;
    for direction in 0..6u32 {
        // Check if the node is in the proper direction relative to parent
        'inner: loop {
            if current.depth() == 0 {
                return 0;
            }

            let index = current.index();
            let parent = current.parent().unwrap();
            let sibling_base = nodes[parent].children().unwrap().get();
            let local_index_relative_to_parent = index - sibling_base;
            let offset = math::CHILDREN_OFFSETS[local_index_relative_to_parent];

            // This "val" needs to either be 0 or 1 depending of div
            let idxdir = (direction as usize) % 3;
            let div = direction / 3;
            let val = offset[idxdir];

            // If div = 0, (start X, start Y, start Z), then val must also be 0
            // If div = 1, (end X, end Y, end Z), then val must also be 1
            let correct = val == div;

            if correct {
                // Find neighbor where all values other
                let sibling_index = math::CHILDREN_OFFSETS.iter().position(|other| {
                    let mut cpy = *other;
                    cpy[idxdir] = offset[idxdir];
                    cpy != offset
                }).unwrap();

                // We found neighbor
                //let neighbor = nodes[sibling_base + sibling_index];

                skirts = u32::MAX;

                break 'inner;
            } else {
                // If we're not in the right direction, move up the tree
                current = &nodes[parent];
            }
        }
    }

    skirts
    */
}
