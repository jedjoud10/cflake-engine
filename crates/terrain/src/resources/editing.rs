use std::{sync::{atomic::AtomicBool, Arc, mpsc::Receiver}, thread::JoinHandle};

use assets::Assets;
use graphics::{
    ActiveComputeDispatcher, BindGroup, Compiler, ComputeModule, ComputeShader, GpuPod, Graphics,
    ModuleVisibility, PushConstantLayout, PushConstants, StorageAccess, Texel,
    Texture3D, Vertex, RG, R,
};
use notify::Watcher;
use crate::{create_texture3d, TerrainSettings};

// Edit handler that will store edited chunk diffs and the main 3D index texture
pub struct EditManager {
    // 128x128x128 texture containing an index to an array of textures
    pub(crate) index_texture: Texture3D<R<u32>>,

    // Multiple textures containing the delta density offsets of each chunk page
    pub(crate) textures: Vec<Texture3D<R<f32>>>,
}

impl EditManager {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        settings: &mut TerrainSettings,
    ) -> Self {
        todo!()
    }
}