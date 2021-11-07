use rendering::Material;

// Terrain settings
pub struct TerrainSettings {
    pub octree_depth: u8,
    pub bound_materials: Vec<Material>,
    pub voxel_generator_interpreter: terrain_interpreter::Interpreter,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            octree_depth: 8,
            bound_materials: vec![Material::default()],
            voxel_generator_interpreter: terrain_interpreter::Interpreter::default(),
        }
    }
}
