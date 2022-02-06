use main::{
    ecs::entity::EntityID,
    globals::Global,
    math::octrees::{DiffOctree, HeuristicSettings},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, AtomicGroupRead, ClearCondition},
            compute::ComputeShader,
            shader_storage::ShaderStorage,
        },
        basics::{
            material::Material,
            readwrite::ReadBytes,
            shader::{self, ShaderSettings},
            transfer::Transferable,
            uniforms::ShaderUniformsGroup,
        },
        object::{ObjectID, ReservedTrackedID, TrackedTask},
        pipeline::{pipec, PipelineContext},
        utils::{AccessType, UpdateFrequency},
    },
    terrain::{ChunkCoords, PackedVoxel, PackedVoxelData, StoredVoxelData, CHUNK_SIZE},
};
use std::{
    collections::{HashMap, HashSet},
    mem::size_of,
};

#[derive(Global)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub chunks_generating: HashSet<ChunkCoords>,
    pub sorted_chunks_generating: Vec<(EntityID, f32)>,
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
    pub compute_id: ReservedTrackedID,
    pub compute_id2: ReservedTrackedID,
    pub read_counters: ReservedTrackedID,
    pub read_final_voxels: ReservedTrackedID,

    // The Entity ID of the chunk that we are generating this voxel data for
    pub chunk_id: Option<EntityID>,
    // And the voxel data for said chunk
    pub packed_chunk_voxel_data: PackedVoxelData,
    pub stored_chunk_voxel_data: StoredVoxelData,
    // We also store the Entity ID of the chunk whom we must create the mesh for
    pub mesh_gen_chunk_id: Option<EntityID>,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(voxel_src_path: &str, octree_depth: u8, pipeline_context: &PipelineContext) -> Self {
        // Create a new octree
        let octree = DiffOctree::new(octree_depth, (CHUNK_SIZE) as u64, HeuristicSettings::default());
        // Load the first pass compute shader
        let pipeline = pipeline_context.read();
        let voxel_src_path = format!("#include {}", format!(r#""{}""#, voxel_src_path));
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path.clone())
            .shader_constant("chunk_size", CHUNK_SIZE);

        let base_compute = ComputeShader::new(settings).unwrap();
        let base_compute = pipec::construct(&pipeline, base_compute).unwrap();

        // Load the second pass compute shader
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path)
            .shader_constant("chunk_size", CHUNK_SIZE);
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipec::construct(&pipeline, second_compute).unwrap();

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
        let reserved_id = ReservedTrackedID::default();
        let info = shader::info::ShaderInfo::default();
        let transfer = info.transfer();
        pipec::tracked_task(&pipeline, TrackedTask::QueryComputeShaderInfo(second_compute, settings, transfer), reserved_id);
        drop(pipeline);

        // Force a pipeline flush and wait till we get the results back
        pipec::flush_and_execute(pipeline_context).unwrap();

        // Now we wait...
        let pipeline = pipeline_context.read();
        while !pipec::did_tasks_execute(&pipeline, &[reserved_id]) {}

        // We got our shader info back!
        let params = info.get(&resource).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] {
            byte_size
        } else {
            panic!()
        };
        let arb_voxels_size = byte_size * ((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
        let params = info.get(&resource2).unwrap();
        let byte_size = if let shader::info::UpdatedParameter::ByteSize(byte_size) = params[0] {
            byte_size
        } else {
            panic!()
        };
        let final_voxel_size = byte_size;
        let final_voxels_size = byte_size * ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1));
        dbg!(final_voxel_size);
        dbg!(size_of::<PackedVoxel>());
        if final_voxel_size != size_of::<PackedVoxel>() {
            panic!()
        }

        // Also construct the atomics
        let atomics = pipec::construct(&pipeline, AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution)).unwrap();

        // Load the shader storage
        let pipeline = pipeline_context.read();
        let shader_storage_arbitrary_voxels = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Pass, arb_voxels_size);
        let shader_storage_arbitrary_voxels = pipec::construct(&pipeline, shader_storage_arbitrary_voxels).unwrap();

        let shader_storage_final_voxels = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Read, final_voxels_size);
        let shader_storage_final_voxels = pipec::construct(&pipeline, shader_storage_final_voxels).unwrap();

        println!("Terrain component init done!");
        Self {
            octree,
            chunks: Default::default(),
            chunks_generating: Default::default(),
            chunks_to_remove: Default::default(),
            packed_chunk_voxel_data: PackedVoxelData::with_voxel_size(),
            stored_chunk_voxel_data: StoredVoxelData::new(),
            sorted_chunks_generating: Default::default(),
            material: ObjectID::default(),
            compute_id: ReservedTrackedID::default(),
            read_counters: ReservedTrackedID::default(),
            compute_id2: ReservedTrackedID::default(),
            read_final_voxels: ReservedTrackedID::default(),
            cpu_data: None,
            chunk_id: None,
            mesh_gen_chunk_id: None,
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
