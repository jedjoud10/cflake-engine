use crate::TerrainSettings;
use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputeShader, DrawIndexedIndirect, GpuPod, Graphics,
    ModuleVisibility, PushConstantLayout, StorageAccess,
};
use rendering::CameraUniform;

// Chunk culler will be responsible for culling invisible / culled (GPU frustum culled) chunks that are not visible
// TODO: Implement some sort of GPU occlusion culling with the "fill" state of each chunk
pub struct ChunkCuller {
    pub(crate) compute_cull: ComputeShader,
}

impl ChunkCuller {
    pub(crate) fn new(assets: &Assets, graphics: &Graphics, _settings: &TerrainSettings) -> Self {
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/cull.comp")
            .unwrap();

        let mut compiler = Compiler::new(assets, graphics);

        compiler.use_push_constant_layout(
            PushConstantLayout::single(u32::size() * 2, ModuleVisibility::Compute).unwrap(),
        );

        compiler.use_storage_buffer::<u32>("visibility", StorageAccess::ReadOnly);
        compiler.use_storage_buffer::<u32>("count", StorageAccess::ReadWrite);
        compiler
            .use_storage_buffer::<vek::Vec4<f32>>("input_position_scale", StorageAccess::ReadOnly);
        compiler.use_storage_buffer::<vek::Vec4<f32>>(
            "output_position_scale",
            StorageAccess::WriteOnly,
        );
        compiler
            .use_storage_buffer::<DrawIndexedIndirect>("input_indirect", StorageAccess::ReadOnly);
        compiler
            .use_storage_buffer::<DrawIndexedIndirect>("output_indirect", StorageAccess::WriteOnly);

        Self {
            compute_cull: ComputeShader::new(module, &compiler).unwrap(),
        }
    }
}
