use main::{
    ecs::{component::Component, entity::EntityID},
    math::octrees::{DiffOctree, HeuristicSettings},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, AtomicGroupRead, ClearCondition},
            compute::ComputeShader,
            shaderstorage::ShaderStorage,
        },
        basics::{
            material::Material,
            readwrite::ReadBytes,
            shader::{self, ShaderSettings},
            transfer::Transferable,
            uniforms::ShaderUniformsGroup,
        },
        object::{ObjectID, PipelineTrackedTask, ReservedTrackedTaskID},
        pipeline::{pipec, PipelineContext},
        utils::{AccessType, UpdateFrequency},
    },
    terrain::{ChunkCoords, Voxel, MAIN_CHUNK_SIZE},
};
use std::{collections::{HashMap, HashSet}, mem::size_of};

#[derive(Component)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub chunks_generating: HashSet<ChunkCoords>,
    pub chunks_to_remove: Vec<EntityID>,
    pub material: ObjectID<Material>,

    // Voxel Generation
    pub custom_uniforms: ShaderUniformsGroup,
    pub compute_shader: ObjectID<ComputeShader>,
    pub second_compute_shader: ObjectID<ComputeShader>,
    pub atomics: ObjectID<AtomicGroup>,
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
}

impl Terrain {
    // Create a new terrain component
    pub fn new(voxel_src_path: &str, octree_depth: u8, pipeline_context: &PipelineContext) -> Self {
        // Create a new octree
        let octree = DiffOctree::new(octree_depth, (MAIN_CHUNK_SIZE) as u64, HeuristicSettings::default());

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
        //pipec::flush_and_execute(pipeline_context).unwrap();

        // Now we wait...
        let pipeline = pipeline_context.read();
        while !pipec::did_tasks_execute(&[reserved_id], &pipeline) {}

        // We got our shader info back!
        let params = info.get(&resource).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] {
            byte_size
        } else {
            panic!()
        };
        let arb_voxels_size = byte_size * ((MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2));
        let params = info.get(&resource2).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] {
            byte_size
        } else {
            panic!()
        };
        let final_voxel_size = byte_size;
        let final_voxels_size = byte_size * ((MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1));
        dbg!(final_voxel_size);
        dbg!(size_of::<Voxel>());
        if final_voxel_size != size_of::<Voxel>() {
            panic!()
        }

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
            chunks_generating: Default::default(),
            chunks_to_remove: Default::default(),
            material: ObjectID::default(),
            compute_id: ReservedTrackedTaskID::default(),
            read_counters: ReservedTrackedTaskID::default(),
            compute_id2: ReservedTrackedTaskID::default(),
            read_final_voxels: ReservedTrackedTaskID::default(),
            cpu_data: None,
            chunk_id: None,
            custom_uniforms: ShaderUniformsGroup::default(),
            compute_shader: base_compute,
            second_compute_shader: second_compute,
            atomics,
            shader_storage_arbitrary_voxels,
            shader_storage_final_voxels,
        }
    }
    // Generate the terrain with a specific material
    pub fn set_material(mut self, material: ObjectID<Material>) -> Self {
        self.material = material;
        self
    }
    // Generate the terrain with a specific octree heuristic settings
    pub fn set_heuristic(mut self, settings: HeuristicSettings) -> Self {
        self.octree.update_heuristic(settings);
        self
    }
    // Generate the terrain with some specific compute shader uniforms
    pub fn set_uniforms(mut self, uniforms: ShaderUniformsGroup) -> Self {
        self.custom_uniforms = uniforms;
        self
    }
}
