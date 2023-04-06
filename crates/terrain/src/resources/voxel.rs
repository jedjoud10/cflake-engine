
use std::rc::Rc;

use assets::Assets;

use graphics::{
    Compiler, ComputeModule, ComputeShader, GpuPod, Graphics, ModuleVisibility, PushConstantLayout, Texel,
    Texture3D, Vertex, R, StorageAccess, RGBA, Normalized, RG, BindGroup,
};



use crate::{TerrainSettings, create_texture3d};

// Voxel generator that will be solely used for generating voxels 
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) voxels: Texture3D<RG<f32>>,
    pub(crate) set_group_callback: Box<dyn Fn(&mut BindGroup) + 'static>,
}

impl VoxelGenerator {
    pub(crate) fn new(assets: &Assets, graphics: &Graphics, settings: &mut TerrainSettings) -> Self {
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/voxels.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);

        // Use the 3D voxels texture that we will write to
        compiler.use_storage_texture::<Texture3D<RG<f32>>>(
            "voxels",
            StorageAccess::WriteOnly
        );

        // Needed by default
        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <vek::Vec4<f32> as GpuPod>::size() + u32::size() * 2,
                ModuleVisibility::Compute,
            )
            .unwrap(),
        );

        let compiler_callback = settings.compiler_callback.take().unwrap();
        let set_group_callback = settings.set_group_callback.take().unwrap();

        // Call the compiler callback
        (compiler_callback)(&mut compiler);

        // Compile the compute shader
        let compute_voxels = ComputeShader::new(module, compiler).unwrap();

        Self {
            compute_voxels,
            voxels: create_texture3d(graphics, settings.size),
            set_group_callback,
        }
    }
}