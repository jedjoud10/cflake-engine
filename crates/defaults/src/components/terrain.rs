use ecs::{ Component, ComponentInternal, ComponentID };
use math::octree::Octree;
use terrain::{CHUNK_SIZE, ChunkManager, Voxel, VoxelGenerator};

// Terrain data that will be on the terrain entity
#[derive(Default)]
pub struct TerrainData {
    pub material: rendering::Material,
    pub octree: Octree,
    pub voxel_generator: VoxelGenerator,
    pub chunk_manager: ChunkManager,    
}

// Create a new terrain data
impl TerrainData {
    // New terrain data with specific parameters
    pub fn new(material: rendering::Material, compute_shader_name: String, octree_depth: u8, lod_factor: f32) -> Self {
        // Create a new octree
        let octree = Octree {
            lod_factor,
            size: (CHUNK_SIZE - 2) as u64,
            depth: octree_depth,
            ..Octree::default()
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

