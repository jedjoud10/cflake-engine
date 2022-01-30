use main::{
    ecs::{component::Component, entity::EntityID},
    math::octrees::{DiffOctree, Octree, OctreeNode, HeuristicSettings},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, ClearCondition, AtomicGroupRead},
            compute::ComputeShader, shaderstorage::ShaderStorage,
        },
        basics::{material::Material, shader::{ShaderSettings, self}, transfer::Transferable, readwrite::ReadBytes},
        object::{ObjectID, ReservedTrackedTaskID, PipelineTrackedTask},
        pipeline::{pipec, Pipeline, PipelineContext}, utils::{UpdateFrequency, AccessType},
    },
    terrain::{ChunkCoords, MAIN_CHUNK_SIZE, Voxel},
};
use std::{collections::HashMap, marker::PhantomData, mem::size_of};


#[derive(Component)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub chunks_to_remove: Vec<EntityID>,
    pub material: ObjectID<Material>,
    pub generating: bool,
    pub swap_chunks: bool,

    // Voxel Generation
    pub compute_shader: ObjectID<ComputeShader>,
    pub second_compute_shader: ObjectID<ComputeShader>,
    // Our 2 shader storages
    pub shader_storage_arbitrary_voxels: ObjectID<ShaderStorage>,
    pub shader_storage_final_voxels: ObjectID<ShaderStorage>,
    // Some CPU side objects that let us retrieve the GPU data
    pub cpu_data: Option<(AtomicGroupRead, ReadBytes)>,
    // The IDs of the generation tasks
    pub compute_id: ReservedTrackedTaskID,
    pub compute_id2: ReservedTrackedTaskID,
    pub read_counters: ReservedTrackedTaskID,
    pub read_final_voxels: ReservedTrackedTaskID,

    // The Entity ID of the chunk that we are generating this voxel data for
    pub chunk_id: Option<EntityID>,    
    pub atomics: ObjectID<AtomicGroup>,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(voxel_src_path: &str, material: ObjectID<Material>, octree_depth: u8, pipeline_context: &PipelineContext) -> Self {
        // Create a new octree
        let octree = DiffOctree::new(octree_depth, (MAIN_CHUNK_SIZE) as u64, HeuristicSettings::new(|node, target| {
            let dist = veclib::Vector3::<f32>::distance(node.get_center().into(), *target) / (node.half_extent as f32 * 2.0);
            dist < 1.2
        }));

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
        
        // We must read the size of the buffer_data Shader Storage Block in the second shader, so we will need to do a pipeline flush
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
        pipec::tracked_task(PipelineTrackedTask::QueryComputeShaderInfo(second_compute, settings, transfer), reserved_id, &pipeline);
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
        let params = info.get(&resource2).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] { byte_size } else { panic!() };
        let final_voxels_size = byte_size;
        let final_voxel_size = final_voxels_size / ((MAIN_CHUNK_SIZE+1)*(MAIN_CHUNK_SIZE+1)*(MAIN_CHUNK_SIZE+1));
        dbg!(final_voxel_size);
        dbg!(size_of::<Voxel>());
        if final_voxel_size != size_of::<Voxel>() { panic!() }

        // Also construct the atomics
        let atomics = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), &pipeline);

        // Load the shader storage
        let pipeline = pipeline_context.read();
        let shader_storage_arbitrary_voxels = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Pass, arb_voxels_size);
        let shader_storage_arbitrary_voxels = pipec::construct(shader_storage_arbitrary_voxels, &pipeline);

        let shader_storage_final_voxels = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Read, final_voxels_size);
        let shader_storage_final_voxels = pipec::construct(shader_storage_final_voxels, &pipeline);

        println!("Terrain component init done!");
        Self {
            octree,
            chunks: Default::default(),
            chunks_to_remove: Default::default(),
            material,
            compute_id: ReservedTrackedTaskID::default(),
            read_counters: ReservedTrackedTaskID::default(),
            compute_id2: ReservedTrackedTaskID::default(),
            read_final_voxels: ReservedTrackedTaskID::default(),
            cpu_data: None,
            generating: false,
            swap_chunks: false,
            chunk_id: None,
            compute_shader: base_compute,
            second_compute_shader: second_compute,
            atomics,
            shader_storage_arbitrary_voxels,
            shader_storage_final_voxels,
        }
    }
}
