use main::{
    ecs::{component::Component, entity::EntityID},
    math::octrees::{AdvancedOctree, Octree, OctreeNode},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, ClearCondition},
            compute::ComputeShader,
        },
        basics::{material::Material, shader::{ShaderSettings, self}, transfer::Transferable},
        object::{ObjectID, ReservedTrackedTaskID, PipelineTrackedTask},
        pipeline::{pipec, Pipeline, PipelineContext},
    },
    terrain::{ChunkCoords, Voxable, MAIN_CHUNK_SIZE},
};
use std::{collections::HashMap, marker::PhantomData, mem::size_of};

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
    pub fn new(voxel_src_path: &str, material: ObjectID<Material>, octree_depth: u8, pipeline: &Pipeline) -> Self {
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
        let pipeline = pipeline_context.read();
        let voxel_src_path = format!("#include {}", format!(r#""{}""#, voxel_src_path));
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path.clone())
            .shader_constant("chunk_size", MAIN_CHUNK_SIZE);

        let base_compute = ComputeShader::new(settings).unwrap();
        let base_compute = pipec::construct(base_compute, &pipeline);

        // Load the second pass compute shader
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path)
            .shader_constant("chunk_size", MAIN_CHUNK_SIZE);
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipec::construct(second_compute, &pipeline);
        
        // We must read the size of the buffer_data Shader Storage Block in the shader, so we will need to do a pipeline flush
        let resource = shader::info::Resource {
            res: shader::info::QueryResource::ShaderStorageBlock,
            name: "arbitrary_voxels".to_string(),
        };
        let resource2 = shader::info::Resource {
            res: shader::info::QueryResource::ShaderStorageBlock,
            name: "output_voxels".to_string(),
        };
        let mut settings = shader::info::ShaderInfoQuerySettings::default();
        settings.query(resource.clone(), vec![shader::info::QueryParameter::ByteSize]);
        settings.query(resource2.clone(), vec![shader::info::QueryParameter::ByteSize]);
        let reserved_id = ReservedTrackedTaskID::default();
        let info = shader::info::ShaderInfo::default();
        let transfer = info.transfer();
        pipec::tracked_task(PipelineTrackedTask::QueryComputeShaderInfo(base_compute, settings, transfer), reserved_id, &pipeline);
        drop(pipeline);

        // Force a pipeline flush and wait till we get the results back
        pipec::flush_and_execute(pipeline_context).unwrap();
        pipec::flush_and_execute(pipeline_context).unwrap();

        // Now we wait...
        let pipeline = pipeline_context.read();
        while !pipec::did_tasks_execute(&[reserved_id], &pipeline) {} 
        
        // We got our shader info back!
        let params = info.get(&resource).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] { byte_size } else { panic!() };
        let arb_voxels_size = byte_size;
        let arb_voxel_size = arb_voxels_size / ((MAIN_CHUNK_SIZE+2)*(MAIN_CHUNK_SIZE+2)*(MAIN_CHUNK_SIZE+2));
        dbg!(byte_size);
        let params = info.get(&resource2).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] { byte_size } else { panic!() };
        dbg!(byte_size);
        let final_voxels_size = byte_size;
        let final_voxel_size = final_voxels_size / ((MAIN_CHUNK_SIZE+1)*(MAIN_CHUNK_SIZE+1)*(MAIN_CHUNK_SIZE+1));

        // We must check if they have the same size
        if size_of::<V>() != final_voxel_size {
            panic!();
        }

        // Also construct the atomics
        let pipeline = pipeline_context.read();
        let atomics = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), &pipeline);

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
