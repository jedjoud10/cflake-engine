use world::{
    math::octrees::HeuristicSettings,
    rendering::{
        basics::{material::Material, uniforms::StoredUniforms},
        pipeline::Handle,
    },
};

// Terrain settings for simplicity
pub struct TerrainSettings {
    // Voxel gen
    pub voxel_src_path: String,
    pub uniforms: Option<StoredUniforms>,

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
            uniforms: Default::default(),
            depth: 4,
            heuristic_settings: Default::default(),
            material: Default::default(),
        }
    }
}
