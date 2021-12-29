use math::octrees::OctreeNode;

use crate::{TModel, VoxelData};

// The data that will be used to store the position/scale of the chunk
#[derive(Default, Clone, Copy, Debug)]
pub struct ChunkCoords {
    pub position: veclib::Vector3<i64>,
    pub center: veclib::Vector3<i64>,
    pub size: u64,
    pub depth: u8,
    pub generation: u32,
}

// Generate the chunk coords from an octree node
impl ChunkCoords {
    // New from chunk coords
    pub fn new(octree_node: &OctreeNode) -> Self {
        Self {
            position: octree_node.position,
            center: octree_node.get_center(),
            size: octree_node.half_extent * 2,
            depth: octree_node.depth,
            generation: octree_node.generation,
        }
    }
}

// Equality tests
impl PartialEq for ChunkCoords {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.depth == other.depth
    }
}
impl Eq for ChunkCoords {}
impl std::hash::Hash for ChunkCoords {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.center.hash(state);
        // We will also hash the depth for good measure
        self.depth.hash(state);
    }
}
// A component that will be added to well... chunks
pub struct Chunk {
    pub coords: ChunkCoords,
    pub voxel_data: Option<Option<VoxelData>>,
}

// Main traits implemented
ecs::impl_component!(Chunk);

impl Chunk {
    // New
    pub fn new(coords: ChunkCoords) -> Self {
        Self { coords, voxel_data: None }
    }
}
