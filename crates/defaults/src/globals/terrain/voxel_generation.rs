use std::mem::size_of;

use main::{
    rendering::{
        advanced::{
            atomic::{AtomicGroup, AtomicGroupRead},
            compute::ComputeShader,
            shader_storage::ShaderStorage,
        },
        basics::{
            readwrite::ReadBytes,
            shader::ShaderSettings,
            uniforms::{SetUniformsCallback, ShaderIDType},
        },
        object::{ObjectID, ReservedTrackedID},
        pipeline::{pipec, Pipeline},
        utils::{AccessType, UpdateFrequency},
    },
    terrain::{PackedVoxel, PackedVoxelData, StoredVoxelData, CHUNK_SIZE},
};

#[derive(Default)]
pub struct VoxelGenerator {
    // Voxel Generation
    pub compute_shader: ObjectID<ComputeShader>,
    pub second_compute_shader: ObjectID<ComputeShader>,
    pub atomics: ObjectID<AtomicGroup>,
    // Our 2 shader storages
    pub shader_storage_arbitrary_voxels: ObjectID<ShaderStorage>,
    pub shader_storage_final_voxels: ObjectID<ShaderStorage>,
    // Some CPU side objects that let us retrieve the GPU data
    pub pending_reads: Option<(AtomicGroupRead, ReadBytes)>,
    // The IDs of the generation tasks
    pub compute_id: ReservedTrackedID,
    pub compute_id2: ReservedTrackedID,
    pub read_counters: ReservedTrackedID,
    pub read_final_voxels: ReservedTrackedID,
    // And the voxel data for said chunk
    pub packed_chunk_voxel_data: PackedVoxelData,
    pub stored_chunk_voxel_data: StoredVoxelData,
    // Some uniforms
    pub uniforms: Option<SetUniformsCallback>,
}

impl VoxelGenerator {
    // Create a new voxel generator
    pub fn new(voxel_src_path: &str, uniforms: Option<SetUniformsCallback>, pipeline: &Pipeline) -> Self {
        // Load the first pass compute shader
        let voxel_src_path = format!(r#"#include "{}""#, voxel_src_path);
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path.clone())
            .shader_constant("chunk_size", CHUNK_SIZE);

        let base_compute = ComputeShader::new(settings).unwrap();
        let base_compute = pipec::construct(pipeline, base_compute).unwrap();

        // Load the second pass compute shader
        let settings = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER)
            .external_code("voxel_include_path", voxel_src_path)
            .shader_constant("chunk_size", CHUNK_SIZE);
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipec::construct(pipeline, second_compute).unwrap();

        // Also construct the atomics
        let atomics = pipec::construct(pipeline, AtomicGroup::new(&[0, 0]).unwrap()).unwrap();

        // Load the shader storage
        let shader_storage_arbitrary_voxels = ShaderStorage::new_using_block(
            UpdateFrequency::Stream,
            AccessType::Pass,
            ShaderIDType::ComputeObjectID(second_compute),
            "arbitrary_voxels",
            (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2),
        );
        let shader_storage_arbitrary_voxels = pipec::construct(pipeline, shader_storage_arbitrary_voxels).unwrap();

        let final_voxels_size = ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) * size_of::<PackedVoxel>();
        let shader_storage_final_voxels = ShaderStorage::new(UpdateFrequency::Stream, AccessType::Read, final_voxels_size);
        let shader_storage_final_voxels = pipec::construct(pipeline, shader_storage_final_voxels).unwrap();

        Self {
            compute_shader: base_compute,
            second_compute_shader: second_compute,
            atomics,
            shader_storage_arbitrary_voxels,
            shader_storage_final_voxels,
            uniforms,
            ..Default::default()
        }
    }
}
