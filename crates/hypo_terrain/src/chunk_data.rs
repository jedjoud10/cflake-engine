use hypo_math::octree;
use hypo_math::octree::OctreeNode;

use super::Voxel;
use super::CHUNK_SIZE;
use super::VoxelGenerator;

// Some chunk data
pub struct ChunkData {
    pub coords: ChunkCoords,
    pub voxels: Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            coords: ChunkCoords::default(),
            voxels: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

impl ChunkData {
    // Create new chunk data from a coord struct
    pub fn new(coords: ChunkCoords) -> Self {
        Self {
            coords,
            voxels: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

// The data that will be used to store the position/scale of the chunk
#[derive(Default, Clone, Debug)]
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
            size: octree_node.half_extent*2,
            depth: octree_node.depth,
        }
    }
}