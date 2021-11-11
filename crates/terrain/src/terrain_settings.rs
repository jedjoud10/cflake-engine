use math::constructive_solid_geometry::CSGTree;
use rendering::Material;

// Terrain settings
pub struct TerrainSettings {
    pub octree_depth: u8,
    pub bound_materials: Vec<Material>,
    
    // Interpreter used to translate the Rust code into GLSL code and compile it at runtime
    pub voxel_generator_interpreter: terrain_interpreter::Interpreter,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            octree_depth: 8,
            bound_materials: vec![Material::default()],
            voxel_generator_interpreter: terrain_interpreter::Interpreter::new(),
        }
    }
}
