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
    // The IDs of the generation tasks
    pub compute: ReservedTrackedTaskID,
    pub read_densities: ReservedTrackedTaskID,
    pub read_counters: ReservedTrackedTaskID,
    pub compute_second: ReservedTrackedTaskID,
    pub read_final_voxels: ReservedTrackedTaskID,

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
    pub fn new(voxel_src_path: &str, material: ObjectID<Material>, octree_depth: u8, pipeline: &main::rendering::pipeline::Pipeline) -> Self {
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

        // Load the first pass compute shader
        let voxel_src_path = format!("#include {}", format!(r#""{}""#, voxel_src_path.to_string()));
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path.clone())
            .shader_constant("chunk_size", MAIN_CHUNK_SIZE);
        let base_compute = ComputeShader::new(settings).unwrap();
        let base_compute = pipec::construct(base_compute, pipeline);

        // Load the second pass compute shader
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path)
            .shader_constant("chunk_size", MAIN_CHUNK_SIZE);
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipec::construct(second_compute, pipeline);

        // Also construct the atomic
        let atomic = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), pipeline);


        println!("Terrain component init done!");
        Self {
            octree,
            chunks: Default::default(),
            material,
            generating: None,
            base_compute,
            second_compute,
            counters: atomic,
        }
    }
}

impl_component!(Terrain);
