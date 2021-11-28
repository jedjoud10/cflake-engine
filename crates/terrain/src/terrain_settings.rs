use rendering::basics::Material;

// Terrain settings
pub struct TerrainSettings {
    pub octree_depth: u8,
    pub material: Material,

    // Interpreter used to translate the Rust code into GLSL code and compile it at runtime
    pub voxel_generator_interpreter: terrain_interpreter::Interpreter,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            octree_depth: 8,
            material: Material::default(),
            voxel_generator_interpreter: terrain_interpreter::Interpreter::new_pregenerated(),
        }
    }
}
