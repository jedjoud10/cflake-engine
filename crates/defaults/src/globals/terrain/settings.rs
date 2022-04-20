use std::num::NonZeroU8;

use world::{
    math::octrees::HeuristicSettings,
    rendering::{basics::material::Material, pipeline::Handle},
};

// Terrain settings for simplicity
pub struct TerrainSettings {
    // Voxel gen
    pub voxel_src_path: String,

    // Octree gen
    pub depth: u8,
    pub heuristic_settings: HeuristicSettings,

    // Mesh generator
    pub material: Handle<Material>,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            voxel_src_path: world::terrain::DEFAULT_TERRAIN_VOXEL_SRC.to_string(),
            depth: 4,
            heuristic_settings: Default::default(),
            material: Default::default(),
        }
    }
}
