use getset::Getters;
use world::{
    rendering::{
        advanced::{
            atomic::AtomicGroup,
            compute::ComputeShader,
            shader_storage::ShaderStorage,
            storages::{DynamicBuffer, StaticBuffer},
        },
        basics::shader::{
            info::{QueryParameter, QueryResource, Resource, ShaderInfoQuerySettings},
            query_info, Directive, ShaderInitSettings,
        },
        pipeline::{Handle, Pipeline},
        utils::{AccessType, UpdateFrequency, UsageType},
    },
    terrain::{editing::PackedEdit, PackedVoxel, PackedVoxelData, StoredVoxelData, CHUNK_SIZE},
};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct VoxelGenerator {
    // Voxel Generation
    pub(crate) primary_compute: Handle<ComputeShader>,
    pub(crate) secondary_compute: Handle<ComputeShader>,
    pub(crate) atomics: AtomicGroup,
    // Our 2 shader storages (for voxel generation)
    pub(crate) ssbo_voxels: ShaderStorage<StaticBuffer<u8>>,
    pub(crate) ssbo_final_voxels: ShaderStorage<StaticBuffer<PackedVoxel>>,
    // And another voxel storage for edits
    pub(crate) ssbo_edits: ShaderStorage<DynamicBuffer<PackedEdit>>,
    // And the voxel data for said chunk
    pub(crate) packed: PackedVoxelData,
    pub(crate) stored: StoredVoxelData,
}

impl VoxelGenerator {
    // Create a new voxel generator
    pub(crate) fn new(voxel_src_path: &str, pipeline: &mut Pipeline) -> Self {
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
            .directive("voxel_include_path", Directive::External(voxel_src_path))
            .directive("chunk_size", Directive::Const(CHUNK_SIZE.to_string()));
        let second_compute = ComputeShader::new(settings).unwrap();
        let second_compute = pipeline.compute_shaders.insert(second_compute);
        let second_compute_program = pipeline.compute_shaders.get(&second_compute).unwrap().program();

        // Usage types
        let readback = UsageType {
            access: AccessType::ServerToClient,
            frequency: UpdateFrequency::WriteManyReadMany,
            dynamic: false,
        };
        let passthrough = UsageType {
            access: AccessType::ServerToServer,
            frequency: UpdateFrequency::WriteManyReadMany,
            dynamic: false,
        };
        let write = UsageType {
            access: AccessType::ClientToServer,
            frequency: UpdateFrequency::WriteManyReadMany,
            dynamic: true,
        };

        // Also construct the atomics
        let atomics = AtomicGroup::new(readback, pipeline);

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

        // Load the shader storage
        let ssbo_voxels = ShaderStorage::<StaticBuffer<u8>>::with_capacity(arbitrary_voxels_size, passthrough, pipeline);
        let ssbo_final_voxels = ShaderStorage::<StaticBuffer<PackedVoxel>>::with_capacity((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1), readback, pipeline);

        // Create a new dynamic shader storage for our terrain edits
        let ssbo_edits = ShaderStorage::<DynamicBuffer<PackedEdit>>::new(Vec::default(), write, pipeline);

        Self {
            primary_compute: base_compute,
            secondary_compute: second_compute,
            atomics,
            ssbo_edits,
            ssbo_voxels,
            ssbo_final_voxels,
            packed: PackedVoxelData::default(),
            stored: StoredVoxelData::default(),
        }
    }
}
