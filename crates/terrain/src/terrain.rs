use assets::AssetManager;
use math::octrees::{AdvancedOctree, Octree, OctreeNode};
use rendering::{AdditionalShader, ComputeShader, Material, Shader};

use crate::{ChunkManager, DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE, TerrainSettings, VoxelGenerator};
// A terrain piece
#[derive(Default)]
pub struct Terrain {
    // Managers / generators
    pub octree: AdvancedOctree,
    pub voxel_generator: VoxelGenerator,
    pub chunk_manager: ChunkManager,
    // Terrain settings
    pub settings: TerrainSettings,
}

impl Terrain {
    // Check if a an already existing node could be subdivided even more
    fn can_node_subdivide_twin(node: &OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let c: veclib::Vector3<f32> = node.get_center().into();
        let max = node.depth == 1 || node.depth == 2;
        let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
        node.children_indices.is_none() && node.depth < max_depth && result
    }
    // New terrain data with specific parameters
    pub fn new(settings: TerrainSettings, asset_manager: &mut AssetManager) -> Self {
        // Create a new octree
        let internal_octree = Octree::new(settings.octree_depth, (MAIN_CHUNK_SIZE) as u64);
        let octree = AdvancedOctree::new(internal_octree, Self::can_node_subdivide_twin);

        // Load the compute shader
        let additional_shader_source = settings.voxel_generator_interpreter.read_glsl().unwrap();
        let compute = Shader::new()
            .set_additional_shader_sources(vec![&additional_shader_source])
            .set_additional_shader(AdditionalShader::Compute(ComputeShader::default()))
            .load_shader(vec![DEFAULT_TERRAIN_COMPUTE_SHADER], asset_manager)
            .unwrap();

        // Finally, create self
        Self {
            octree,
            voxel_generator: VoxelGenerator::new(compute),
            chunk_manager: ChunkManager::default(),
            settings,
        }
    }
}
