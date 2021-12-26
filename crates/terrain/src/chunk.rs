use math::octrees::OctreeNode;

use crate::{TModel, VoxelGenerator};

// The data that will be used to store the position/scale of the chunk
#[derive(Default, Clone, Copy, Debug)]
pub struct ChunkCoords {
    pub position: veclib::Vector3<i64>,
    pub center: veclib::Vector3<i64>,
    pub size: u64,
    pub depth: u8,
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
        }
    }
}


// A component that will be added to well... chunks
#[derive(Default)]
pub struct Chunk {
    pub coords: ChunkCoords,
    pub generated: bool,
    pub tmodel: TModel
}

// Main traits implemented
ecs::impl_component!(Chunk);

impl Chunk {
    // When this chunk is created, we must tell the voxel generator to generate the voxel data
    pub fn new(coords: ChunkCoords, voxel_generator: &mut VoxelGenerator) -> Self {        
        Self {
            coords,
            generated: false,
            tmodel: TModel {
                model: None,
                skirts_model: None,
                coords,
            },
        }
        // Tell the voxel generator that it must generated the model for this specific Chunk
    }
}