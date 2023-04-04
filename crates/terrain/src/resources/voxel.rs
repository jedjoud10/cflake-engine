
use assets::Assets;

use graphics::{
    Compiler, ComputeModule, ComputeShader, GpuPod, Graphics, ModuleVisibility, PushConstantLayout, Texel,
    Texture3D, Vertex, R, StorageAccess,
};



use crate::{TerrainSettings, create_texture3d};

// Voxel generator that will be solely used for generating voxels 
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) densities: Texture3D<R<f32>>,
}

impl VoxelGenerator {
    pub(crate) fn new(assets: &Assets, graphics: &Graphics, settings: &TerrainSettings) -> Self {
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/voxels.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);

        // Use the 3D densities texture that we will write to
        compiler.use_storage_texture::<Texture3D<R<f32>>>(
            "densities",
            StorageAccess::WriteOnly
        );

        // TODO: Create 3D color texture as well

        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <vek::Vec4<f32> as GpuPod>::size() + u32::size() * 2,
                ModuleVisibility::Compute,
            )
            .unwrap(),
        );

        // Compile the compute shader
        let compute_voxels = ComputeShader::new(module, compiler).unwrap();

        Self {
            compute_voxels,
            densities: create_texture3d(graphics, settings.size),
        }
    }
}