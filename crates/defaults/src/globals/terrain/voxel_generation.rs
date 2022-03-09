use std::mem::size_of;

use world::{
    rendering::{
        advanced::{
            atomic::AtomicGroup,
            compute::ComputeShader,
            raw::{dynamic::DynamicBuffer, simple::SimpleBuffer},
            shader_storage::ShaderStorage,
        },
        basics::{
            shader::{
                info::{QueryParameter, QueryResource, Resource, ShaderInfoQuerySettings},
                query_info, Directive, ShaderInitSettings,
            },
            uniforms::StoredUniforms,
        },
        pipeline::{Handle, Pipeline},
        utils::{AccessType, UpdateFrequency, UsageType},
    },
    terrain::{editing::PackedEdit, PackedVoxel, PackedVoxelData, StoredVoxelData, CHUNK_SIZE},
};

pub struct VoxelGenerator {
    // Voxel Generation
    pub compute_shader: Handle<ComputeShader>,
    pub second_compute_shader: Handle<ComputeShader>,
    pub atomics: AtomicGroup,
    // Our 2 shader storages (for voxel generation)
    pub shader_storage_arbitrary_voxels: ShaderStorage<SimpleBuffer<u8>, u8>,
    pub shader_storage_final_voxels: ShaderStorage<SimpleBuffer<PackedVoxel>, PackedVoxel>,
    // And another voxel storage for edits
    pub shader_storage_edits: ShaderStorage<DynamicBuffer<PackedEdit>, PackedEdit>,
    pub packed_edits_update: Option<Vec<PackedEdit>>,
    pub packed_edits_num: usize,
    // And the voxel data for said chunk
    pub packed_chunk_voxel_data: PackedVoxelData,
    pub stored_chunk_voxel_data: StoredVoxelData,
    // Some uniforms
    pub uniforms: Option<StoredUniforms>,
}

impl VoxelGenerator {
    // Create a new voxel generator
    pub(crate) fn new(voxel_src_path: &str, uniforms: Option<StoredUniforms>, pipeline: &mut Pipeline) -> Self {
        // Load the first pass compute shader
        let voxel_src_path = format!(r#"#include "{}""#, voxel_src_path);
        let settings = ShaderInitSettings::default()
            .source(world::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER)
            .directive("voxel_include_path", Directive::External(voxel_src_path.to_string()))
            .directive("chunk_size", Directive::Const(CHUNK_SIZE.to_string()));
        let base_compute = ComputeShader::new(settings).unwrap();
        let base_compute = pipeline.compute_shaders.insert(base_compute);

        // Load the second pass compute shader
        let settings = ShaderInitSettings::default()
            .source(world::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER)
            .directive("voxel_include_path", Directive::External(voxel_src_path.to_string()))
            .directive("chunk_size", Directive::Const(CHUNK_SIZE.to_string()));
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipeline.compute_shaders.insert(second_compute);
        let second_compute_program = pipeline.compute_shaders.get(&second_compute).unwrap().program();

        // Also construct the atomics
        let atomics = AtomicGroup::new(UsageType::new(AccessType::ServerToClient, UpdateFrequency::Stream), pipeline);

        // Get the size of each arbitrary voxel
        let mut settings = ShaderInfoQuerySettings::default();
        let resource = Resource {
            res: QueryResource::ShaderStorageBlock,
            name: "arbitrary_voxels".to_string(),
        };
        settings.query(resource.clone(), vec![QueryParameter::ByteSize]);

        // Query
        let shader_info = query_info(second_compute_program, pipeline, settings).unwrap();

        // Read back the byte size
        let byte_size = shader_info.get(&resource).unwrap().get(0).unwrap().as_byte_size().unwrap();

        let arbitrary_voxels_size = byte_size.next_power_of_two() * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2);

        // Usage
        let usage = UsageType::new(AccessType::ServerToServer, UpdateFrequency::Stream);
        let usage2 = UsageType::new(AccessType::ServerToClient, UpdateFrequency::Stream);
        let usage3 = UsageType::new(AccessType::ClientToServer, UpdateFrequency::Dynamic);

        // Load the shader storage
        let shader_storage_arbitrary_voxels = ShaderStorage::<SimpleBuffer<u8>, u8>::new(Vec::with_capacity(arbitrary_voxels_size), usage, pipeline);
        let shader_storage_final_voxels = ShaderStorage::<SimpleBuffer<PackedVoxel>, PackedVoxel>::new(Vec::with_capacity((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)), usage2, pipeline);

        // Create a new dynamic shader storage for our terrain edits
        let shader_storage_edits = ShaderStorage::<DynamicBuffer<PackedEdit>, PackedEdit>::new(Vec::default(), usage3, pipeline);

        Self {
            compute_shader: base_compute,
            second_compute_shader: second_compute,
            atomics,
            shader_storage_edits,
            shader_storage_arbitrary_voxels,
            shader_storage_final_voxels,
            uniforms,
            packed_edits_update: None,
            packed_edits_num: 0,
            packed_chunk_voxel_data: PackedVoxelData::default(),
            stored_chunk_voxel_data: StoredVoxelData::default(),
        }
    }
}
