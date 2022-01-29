use main::{
    ecs::{component::ComponentID, entity::EntityID, impl_component},
    math::{
        self,
        octrees::{AdvancedOctree, Octree, OctreeNode},
    },
    rendering::{
        advanced::{
            atomic::{AtomicGroup, AtomicGroupRead, ClearCondition},
            compute::ComputeShader,
        },
        basics::{
            material::Material,
            shader::ShaderSettings,
            texture::{Texture, TextureFilter, TextureFormat, TextureType, TextureWrapping},
        },
        object::{ObjectID, ReservedTrackedTaskID},
        pipeline::pipec,
        utils::DataType,
    },
    terrain::{ChunkCoords, VoxelData, MAIN_CHUNK_SIZE},
};
use std::collections::HashMap;

// Some data that we store whenever we are generating the voxels
pub struct TerrainGenerationData {
    // The ID of the main tracked task
    pub main_id: ReservedTrackedTaskID,
    // The Entity ID of the chunk that we are generating this voxel data for
    pub chunk_id: EntityID,
}

// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: AdvancedOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub material: ObjectID<Material>,

    // Voxel Generation
    pub generating: Option<TerrainGenerationData>,
    pub base_compute: ObjectID<ComputeShader>,
    pub second_compute: ObjectID<ComputeShader>,
    // Atomics
    pub counters: ObjectID<AtomicGroup>,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(material: ObjectID<Material>, octree_depth: u8, pipeline: &main::rendering::pipeline::Pipeline) -> Self {
        // Check if a an already existing node could be subdivided even more
        fn can_node_subdivide_twin(node: &OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
            let c: veclib::Vector3<f32> = node.get_center().into();
            let max = node.depth == 1 || node.depth == 2;
            let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
            node.children_indices.is_none() && node.depth < max_depth && result
        }
        // Create a new octree
        let internal_octree = Octree::new(octree_depth, (MAIN_CHUNK_SIZE) as u64);
        let octree = AdvancedOctree::new(internal_octree, can_node_subdivide_twin);

        // Load the compute shader
        let ss = ShaderSettings::default().source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER);
        let base_compute = ComputeShader::new(ss).unwrap();
        let base_compute = pipec::construct(base_compute, pipeline);

        let ss = ShaderSettings::default().source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER);
        let second_compute = ComputeShader::new(ss).unwrap();
        let second_compute = pipec::construct(second_compute, pipeline);
        todo!()
    }
}

impl_component!(Terrain);
