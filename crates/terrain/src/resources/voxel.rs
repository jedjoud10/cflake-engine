use std::{sync::{atomic::AtomicBool, Arc, mpsc::Receiver}, thread::JoinHandle};

use assets::Assets;
use graphics::{
    ActiveComputeDispatcher, BindGroup, Compiler, ComputeModule, ComputeShader, GpuPod, Graphics,
    ModuleVisibility, PushConstantLayout, PushConstants, StorageAccess, Texel,
    Texture3D, Vertex, RG,
};
use notify::Watcher;


use crate::{create_texture3d, TerrainSettings};

// The voxel texture will contain all the data that will be serialized and stored/sent over the network
// This is the layout for each texel within said voxel texture
// Bytes 0..2: 16 bit floating-point density

// Voxel generator that will be solely used for generating voxels
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) voxel_textures: [Texture3D<RG<f32>>; 2],
    pub(crate) set_bind_group_callback: Option<Box<dyn Fn(&mut BindGroup) + 'static>>,
    pub(crate) set_push_constant_callback:
        Option<Box<dyn Fn(&mut PushConstants<ActiveComputeDispatcher>) + 'static>>,
    pub(crate) hot_reload: Option<(Receiver<()>, JoinHandle<()>)>,
}

impl VoxelGenerator {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        settings: &mut TerrainSettings,
    ) -> Self {
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/voxels.comp")
            .unwrap();
        let mut compiler = create_compute_voxels_compiler(assets, graphics);
        let compute_voxels = ComputeShader::new(module, &compiler).unwrap();

        /*
        let compiler_callback = settings.voxel_compiler_callback.take();
        let set_bind_group_callback = settings.voxel_set_group_callback.take();
        let set_push_constant_callback = settings.voxel_set_push_constants_callback.take();

        // Call the compiler callback
        if let Some(callback) = compiler_callback {
            (callback)(&mut compiler);
        }
        */

        // Create two textures that will be swapped out every other frame
        let voxel_textures = [
            create_texture3d(graphics, settings.size),
            create_texture3d(graphics, settings.size),
        ];

        // Create a watcher that will watch the voxels compute shader file for any changes
        let hot_reload = if !assets.packed() {
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            let path = assets.path("engine/shaders/terrain/voxel.glsl").unwrap();

            let handle = std::thread::spawn(move || {
                let mut watcher = notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = event {
                        match event.kind {
                            notify::EventKind::Modify(_) => { tx.send(()); },
                            _ => {}
                        }
                    }
                }).unwrap();
                watcher.watch(&path, notify::RecursiveMode::NonRecursive).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
            });
            
            Some((rx, handle))
        } else {
            None
        };
    
        Self {
            compute_voxels,
            voxel_textures,
            set_bind_group_callback: None,
            set_push_constant_callback: None,
            hot_reload,
        }
    }
}

pub(crate) fn create_compute_voxels_compiler<'a>(assets: &'a Assets, graphics: &'a Graphics) -> Compiler<'a> {
    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);

    // Use the 3D voxels texture that we will write to
    compiler.use_storage_texture::<Texture3D<RG<f32>>>("voxels", StorageAccess::WriteOnly);

    // Needed by default
    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <vek::Vec4<f32> as GpuPod>::size() + f32::size() + f32::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );
    compiler
}
