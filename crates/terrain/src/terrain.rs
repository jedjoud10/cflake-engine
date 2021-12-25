use math::{
    constructive_solid_geometry::CSGTree,
    octrees::{AdvancedOctree, Octree, OctreeNode},
};
use rendering::{basics::Shader, pipec};

use crate::{ChunkManager, TerrainSettings, VoxelGenerator, DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE};
// A terrain piece
#[derive(Default)]
pub struct Terrain {
    // Managers / generators
    pub octree: AdvancedOctree,
    pub voxel_generator: VoxelGenerator,
    pub chunk_manager: ChunkManager,
    // Terrain settings
    pub settings: TerrainSettings,
    // CSG Tree
    pub csgtree: CSGTree,
}
ecs::impl_systemdata!(Terrain);

impl Terrain {
    // Check if a an already existing node could be subdivided even more
    fn can_node_subdivide_twin(node: &OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let c: veclib::Vector3<f32> = node.get_center().into();
        let max = node.depth == 1 || node.depth == 2;
        let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
        node.children_indices.is_none() && node.depth < max_depth && result
    }
    // New terrain data with specific parameters
    pub fn new(mut settings: TerrainSettings) -> Self {
        // Create a new octree
        let internal_octree = Octree::new(settings.octree_depth, (MAIN_CHUNK_SIZE) as u64);
        let octree = AdvancedOctree::new(internal_octree, Self::can_node_subdivide_twin);

        // Load the compute shader
        let (string, csgtree) = settings.voxel_generator_interpreter.finalize().unwrap();
        let compute = pipec::compute_shader(
            Shader::default()
                .load_externalcode("voxel_interpreter", string)
                .load_shader(vec![DEFAULT_TERRAIN_COMPUTE_SHADER])
                .unwrap(),
        );
        panic!();

        // Finally, create self
        Self {
            octree,
            voxel_generator: VoxelGenerator::new(compute),
            chunk_manager: ChunkManager::default(),
            csgtree: csgtree,
            settings,
        }
    }
}
