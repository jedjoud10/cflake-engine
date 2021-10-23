use ecs::{Component, ComponentID, ComponentInternal};
use math::octrees::*;
use terrain::{ChunkManager, VoxelGenerator, CHUNK_SIZE};

// Terrain data that will be on the terrain entity
#[derive(Default)]
pub struct TerrainData {
    pub material: rendering::Material,
    pub octree: AdvancedOctree,
    pub voxel_generator: VoxelGenerator,
    pub chunk_manager: ChunkManager,
}

// Create a new terrain data
impl TerrainData {
    // Check if a an already existing node could be subdivided even more
    fn can_node_subdivide_twin(node: &OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let c: veclib::Vector3<f32> = node.get_center().into();
        let max = (node.depth == 1 || node.depth == 2) || (c.distance(*target) < 400.0 && node.depth == 3);
        let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
        node.children_indices.is_none() && node.depth < max_depth && result
    }
    // New terrain data with specific parameters
    pub fn new(material: rendering::Material, compute_id: usize, color_compute_id: usize, octree_depth: u8) -> Self {
        // Create a new octree
        let mut octree = AdvancedOctree {
            internal_octree: Octree {
                depth: octree_depth,
                size: (CHUNK_SIZE - 2) as u64,
                ..Octree::default()
            },
            ..AdvancedOctree::default()
        };
        // Add the twin rule
        octree.set_twin_generation_rule(Self::can_node_subdivide_twin);
        Self {
            material,
            octree,
            voxel_generator: VoxelGenerator {
                compute_id: compute_id,
                color_compute_id: color_compute_id,
                ..VoxelGenerator::default()
            },
            chunk_manager: ChunkManager::default(),
        }
    }
}

ecs::impl_component!(TerrainData);
