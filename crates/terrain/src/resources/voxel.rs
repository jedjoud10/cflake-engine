use std::rc::Rc;
use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputeShader, GpuPod, Graphics, ModuleVisibility, PushConstantLayout, Texel,
    Texture3D, Vertex, R, StorageAccess, RGBA, Normalized, RG, BindGroup, PushConstants, ActiveComputeDispatcher,
};

use crate::{TerrainSettings, create_texture3d};

// Voxel generator that will be solely used for generating voxels 
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) voxels: Texture3D<RG<f32>>,
    pub(crate) set_bind_group_callback: Option<Box<dyn Fn(&mut BindGroup) + 'static>>,
    pub(crate) set_push_constant_callback: Option<Box<dyn Fn(&mut PushConstants<ActiveComputeDispatcher>) + 'static>>
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

        let compiler_callback = settings.voxel_compiler_callback.take();
        let set_bind_group_callback = settings.voxel_set_group_callback.take();
        let set_push_constant_callback = settings.voxel_set_push_constants_callback.take();

        // Call the compiler callback
        if let Some(callback) = compiler_callback {
            (callback)(&mut compiler);
        }

        // Compile the compute shader
        let compute_voxels = ComputeShader::new(module, compiler).unwrap();

        Self {
            compute_voxels,
            voxels: create_texture3d(graphics, settings.size),
            set_bind_group_callback,
            set_push_constant_callback,
        }
    }
}