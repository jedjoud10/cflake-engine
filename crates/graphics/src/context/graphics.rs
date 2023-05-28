use ahash::AHashMap;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::{hash::BuildHasherDefault, path::PathBuf, sync::Arc};
use thread_local::ThreadLocal;
use utils::Storage;
pub use wgpu::CommandEncoder;
use wgpu::{
    util::StagingBelt, Adapter, Device, Instance, Maintain, Queue, Sampler, Surface,
    SurfaceCapabilities, SurfaceConfiguration, TextureView,
};

use crate::{
    BindGroupLayout, BindResourceLayout, Id, ReflectedShader, SamplerSettings, SamplerWrap,
    Snippets, StagingPool, UniformBuffer, Defines, CachedSpirvKey, CachedShaderKey,
};

// Cached graphics data that can be reused
pub(crate) struct Cached {
    pub(crate) spirvs: 
        DashMap<CachedSpirvKey, Vec<u32>>,
    pub(crate) shaders:
        DashMap<CachedShaderKey, (Arc<wgpu::ShaderModule>, Arc<spirq::EntryPoint>)>,
    pub(crate) samplers: DashMap<SamplerSettings, Arc<Sampler>>,
    pub(crate) bind_group_layouts: DashMap<BindGroupLayout, Arc<wgpu::BindGroupLayout>>,
    pub(crate) pipeline_layouts: DashMap<ReflectedShader, Arc<wgpu::PipelineLayout>>,
    pub(crate) bind_groups: DashMap<Vec<Id>, Arc<wgpu::BindGroup>>,
}

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    // Main WGPU instance, device, and shenanigans
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,

    // List of command encoders that are unused per thread
    pub(crate) encoders: ThreadLocal<Mutex<Vec<CommandEncoder>>>,

    // Helpers and cachers
    pub(crate) staging: StagingPool,
    pub(crate) shaderc: shaderc::Compiler,
    pub(crate) cached: Cached,

    // Keep track of these numbers for statistics
    pub(crate) acquires: Mutex<u32>,
    pub(crate) submissions: Mutex<u32>,
    pub(crate) stalls: Mutex<u32>,
}

// Stats that can be displayed using egui
#[derive(Default, Clone, Copy)]
pub struct GraphicsStats {
    pub acquires: usize,
    pub submissions: usize,
    pub stalls: usize,
    pub staging_buffers: usize,

    pub cached_shaders: usize,
    pub cached_samplers: usize,
    pub cached_bind_group_layouts: usize,
    pub cached_pipeline_layouts: usize,
    pub cached_bind_groups: usize,

    pub adapters: usize,
    pub devices: usize,
    pub pipeline_layouts: usize,
    pub shader_modules: usize,
    pub bind_group_layouts: usize,
    pub bind_groups: usize,
    pub command_buffers: usize,
    pub render_pipelines: usize,
    pub compute_pipelines: usize,
    pub buffers: usize,
    pub textures: usize,
    pub texture_views: usize,
    pub samplers: usize,
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(crate) Arc<InternalGraphics>);

impl Graphics {
    // Get the internally stored instance
    pub fn instance(&self) -> &Instance {
        &self.0.instance
    }

    // Get the internally stored device
    pub fn device(&self) -> &Device {
        &self.0.device
    }

    // Get the internally stored queue
    pub fn queue(&self) -> &Queue {
        &self.0.queue
    }

    // Get the GPU we are using
    pub fn adapter(&self) -> &Adapter {
        &self.0.adapter
    }

    // Get the global staging buffer allocator
    pub fn staging_pool(&self) -> &StagingPool {
        &self.0.staging
    }

    // Create a new command encoder to record commands
    // This might fetch an already existing command encoder (for this thread), or it will create a new one
    pub fn acquire(&self) -> CommandEncoder {
        log::trace!("graphics context: acquire");
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        *self.0.acquires.lock() += 1;
        locked.pop().unwrap_or_else(|| {
            self.device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
        })
    }

    // Submit one or multiple command encoders and possibly waits for the GPU to complete them
    // The submitted command encoders cannot be reused for new commands
    pub fn submit_from_iter(&self, iter: impl IntoIterator<Item = CommandEncoder>, wait: bool) {
        log::trace!("graphics context: submit from iter");
        let finished = iter.into_iter().map(|x| x.finish());
        let i = self.queue().submit(finished);
        *self.0.submissions.lock() += 1;

        if wait {
            self.device().poll(Maintain::WaitForSubmissionIndex(i));
            *self.0.stalls.lock() += 1;
        }
    }

    // Submit all the currently unused command encoders and clears the thread local cache
    pub fn submit(&self, wait: bool) {
        log::trace!("graphics context: submit, wait: {wait}");
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        self.submit_from_iter(locked.drain(..), wait);
    }

    // Pushes some unfinished command encoders to be re-used by the current thread
    pub fn reuse(&self, iter: impl IntoIterator<Item = CommandEncoder>) {
        log::trace!("graphics context: reuse");
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.extend(iter);
    }

    // Called internally when we drop a resource to free it from the cached bind groups
    pub(crate) fn drop_cached_bind_group_resource(&self, id: Id) {
        let mut keys_to_remove = Vec::<Vec<Id>>::default();

        for pair in self.0.cached.bind_groups.iter() {
            if pair.key().contains(&id) {
                keys_to_remove.push(pair.key().to_vec());
            }
        }

        for keys in keys_to_remove {
            self.0.cached.bind_groups.remove(&keys);
        }
    }

    // Called internally when we drop a render shader or compute shader pipeline layout (and it's corresponding shared bind group layouts)
    // TODO: Should we even remove the bind group layouts in the first place??
    pub(crate) fn drop_cached_pipeline_layout(&self, reflected: &ReflectedShader) -> bool {
        let cached = &self.0.cached;
        
        let bind_group_layouts = reflected.bind_group_layouts.iter().filter_map(|x| x.as_ref());

        for bind_group_layout in bind_group_layouts {
            let remove_bind_group_layout = cached.bind_group_layouts
                .get(bind_group_layout)
                .map(|x| Arc::strong_count(x.value()) == 1)
                .unwrap_or_default();

            if remove_bind_group_layout {
                cached.bind_group_layouts.remove(bind_group_layout).unwrap();
            }
        }

        self.0.cached.pipeline_layouts.remove(&reflected).is_some()
    }

    pub(crate) fn drop_cached_shader_module(&self, key: CachedShaderKey) -> bool {
        let cached = &self.0.cached;
        cached.shaders.remove(&key).is_some()
    }

    pub(crate) fn drop_cached_spirv(&self, key: CachedSpirvKey) -> bool {
        let cached = &self.0.cached;
        cached.spirvs.remove(&key).is_some()
    } 
}
