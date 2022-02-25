use world::{
    math::octrees::HeuristicSettings,
    rendering::{
        basics::{material::Material, uniforms::SetUniformsCallback},
        object::ObjectID,
    },
};

// Terrain settings for simplicity
pub struct TerrainSettings {
    // Voxel gen
    pub(crate) voxel_src_path: String,
    pub(crate) uniforms: Option<SetUniformsCallback>,

    // Octree gen
    pub(crate) depth: u8,
    pub(crate) heuristic_settings: HeuristicSettings,

    // Mesh generator
    pub(crate) material: ObjectID<Material>,
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

impl TerrainSettings {
    // Initialize the voxel generator with a specific voxel source
    pub fn with_voxel_src(mut self, path: &str) -> Self {
        self.voxel_src_path = path.to_string();
        self
    }
    // Set the depth for the octree
    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }
    // Generate the chunks with a specific material
    pub fn with_material(mut self, material: ObjectID<Material>) -> Self {
        self.material = material;
        self
    }
    // Generate the terrain with a specific octree heuristic settings
    pub fn with_heuristic(mut self, settings: HeuristicSettings) -> Self {
        self.heuristic_settings = settings;
        self
    }
    // Generate the terrain with some specific uniforms
    pub fn with_uniforms(mut self, uniforms: SetUniformsCallback) -> Self {
        self.uniforms = Some(uniforms);
        self
    }
}
