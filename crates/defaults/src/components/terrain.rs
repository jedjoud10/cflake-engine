use ecs::{ Component, ComponentInternal, ComponentID };
use math::octrees::*;
use terrain::{CHUNK_SIZE, ChunkManager, Voxel, VoxelGenerator};

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
    // New terrain data with specific parameters
    pub fn new(material: rendering::Material, compute_shader_name: String, octree_depth: u8) -> Self {
        // Create a new octree
        let octree = AdvancedOctree {
            internal_octree: Octree { depth: octree_depth, size: (CHUNK_SIZE - 2) as u64, ..Octree::default() },
            ..AdvancedOctree::default()
        };
        Self {
            material,
            octree,
            voxel_generator: VoxelGenerator {
                compute_shader_name,
                ..VoxelGenerator::default()
            },
            chunk_manager: ChunkManager::default(),
        }        
    }
}

ecs::impl_component!(TerrainData);

