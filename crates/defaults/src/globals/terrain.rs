use main::{
    ecs::{component::Component, entity::EntityID},
    math::octrees::{AdvancedOctree, Octree, OctreeNode},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, ClearCondition},
            compute::ComputeShader,
        },
        basics::{material::Material, shader::ShaderSettings},
        object::{ObjectID, ReservedTrackedTaskID},
        pipeline::pipec,
    },
    terrain::{ChunkCoords, Voxable, MAIN_CHUNK_SIZE},
};
use std::{collections::HashMap, marker::PhantomData};

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

#[derive(Component)]
// The global terrain component that can be added at the start of the game
pub struct Terrain<U: Voxable + 'static> {
    // Chunk generation
    pub octree: AdvancedOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub material: ObjectID<Material>,

    // Voxel Generation
    pub generating: Option<TerrainGenerationData>,
    pub base_compute: ObjectID<ComputeShader>,
    pub second_compute: ObjectID<ComputeShader>,
    //pub shader_storage: ObjectID<ShaderStorage>,
    pub atomics: ObjectID<AtomicGroup>,

    _phantom: PhantomData<U>,
}

impl<V: Voxable + 'static> Terrain<V> {
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
        let voxel_src_path = format!("#include {}", format!(r#""{}""#, voxel_src_path));
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
        /*
        const int _CHUNK_SIZE = #constant chunk_size
        const int _CSPO = _CHUNK_SIZE + 1; // Chunk size plus one
        const int _CSPT = _CHUNK_SIZE + 2; // Chunk size plus two
        // Load the voxel function file
        layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
        layout(binding = 2) uniform atomic_uint positive_counter;
        layout(binding = 2) uniform atomic_uint negative_counter;
        layout(std430, binding = 3) buffer buffer_data
        {
            Voxel voxels[_CSPT][_CSPT][_CSPT];
            BundledVoxel bundled_voxels[_CSPO][_CSPO][_CSPO];
        };


        // Create a Shader Storage that will hold all of our voxel data
        let arbitrary_data = (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) *
        let shader_storage = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Read, )
        */

        // Also construct the atomics
        let atomics = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), pipeline);

        println!("Terrain component init done!");
        Self {
            octree,
            chunks: Default::default(),
            material,
            generating: None,
            base_compute,
            second_compute,
            atomics,
            _phantom: PhantomData::default(),
        }
    }
}
