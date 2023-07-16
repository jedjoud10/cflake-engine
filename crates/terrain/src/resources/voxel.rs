use std::{sync::mpsc::Receiver, thread::JoinHandle};

use crate::{create_texture3d, TerrainSettings};
use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputeShader, GpuPod, Graphics, ModuleVisibility, PushConstantLayout,
    SamplerSettings, SamplerWrap, StorageAccess, Texel, Texture3D, TextureUsage, Vertex, RG,
};
use notify::Watcher;

// Voxel generator that will be solely used for generating voxels
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) voxel_texture: Texture3D<RG<f32>>,
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
        let compiler = create_compute_voxels_compiler(assets, graphics);
        let compute_voxels = ComputeShader::new(module, &compiler).unwrap();

        // Create the main voxel texture
        let voxel_textures = create_texture3d(
            graphics,
            settings.mesher.size,
            TextureUsage::STORAGE | TextureUsage::WRITE | TextureUsage::SAMPLED,
            Some(SamplerSettings {
                wrap_u: SamplerWrap::ClampToEdge,
                wrap_v: SamplerWrap::ClampToEdge,
                wrap_w: SamplerWrap::ClampToEdge,
                ..Default::default()
            }),
        );

        // Create a watcher that will watch the voxels compute shader file for any changes
        let hot_reload = if !assets.packed() {
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            let path = assets.path("engine/shaders/terrain/voxel.glsl").unwrap();

            let handle = std::thread::spawn(move || {
                let mut watcher = notify::recommended_watcher(
                    move |event: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = event {
                            match event.kind {
                                notify::EventKind::Modify(_) => {
                                    tx.send(());
                                }
                                _ => {}
                            }
                        }
                    },
                )
                .unwrap();
                watcher
                    .watch(&path, notify::RecursiveMode::NonRecursive)
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
            });

            Some((rx, handle))
        } else {
            None
        };

        Self {
            compute_voxels,
            voxel_texture: voxel_textures,
            hot_reload,
        }
    }
}

pub(crate) fn create_compute_voxels_compiler<'a>(
    assets: &'a Assets,
    graphics: &'a Graphics,
) -> Compiler<'a> {
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
