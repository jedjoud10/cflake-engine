use ecs::{ Component, ComponentInternal, ComponentID };
use math::octree::Octree;
use terrain::ChunkManager;

// Terrain data that will be on the terrain entity
#[derive(Default)]
pub struct TerrainData {
    // The material for this terrain
    pub material: rendering::Material,
    // The octree depth for this terrain
    pub octree_depth: u8,
    // The octree
    pub octree: Octree,
    // The chunk manager
    pub chunk_manager: ChunkManager,
    // The LOD factor for this terrain
    pub lod_factor: f32
}

// Create a new terrain data
impl TerrainData {
    // New terrain data with specific parameters
    pub fn new(material: rendering::Material, shader_name: &str, octree_depth: u8, lod_factor: f32) -> Self {
        Self {
            material,
            octree: Octree::default(),
            chunk_manager: ChunkManager::default(),
            octree_depth,
            lod_factor,
        }        
    }
}

ecs::impl_component!(TerrainData);

