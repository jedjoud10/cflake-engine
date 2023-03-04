use ahash::AHashMap;
use dashmap::DashMap;
use naga::{valid::Validator};
use parking_lot::Mutex;
use thread_local::ThreadLocal;
use std::{hash::BuildHasherDefault, sync::Arc};
use utils::Storage;
use wgpu::{
    util::StagingBelt, Adapter, Device, Queue,
    Sampler, Surface, SurfaceCapabilities, SurfaceConfiguration,
    TextureView, Maintain,
};
pub use wgpu::CommandEncoder;

use crate::{
    BindGroupLayout, ReflectedShader, SamplerSettings, SamplerWrap,
    StagingPool, UniformBuffer, BindEntryLayout,
};


// Cached graphics data
pub(crate) struct Cached {
    pub(crate) samplers: DashMap<SamplerSettings, Arc<Sampler>>,
    pub(crate) bind_group_layouts:
        DashMap<BindGroupLayout, Arc<wgpu::BindGroupLayout>>,
    pub(crate) pipeline_layouts:
        DashMap<ReflectedShader, Arc<wgpu::PipelineLayout>>,
    pub(crate) bind_groups:
        DashMap<Vec<wgpu::Id>, Arc<wgpu::BindGroup>>,
    pub(crate) uniform_buffers:
        Mutex<AHashMap<(u32, BindEntryLayout), Vec<(UniformBuffer<u8>, bool)>>>,
}

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    // Device and queue
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,

    // List of command encoders that are unused per thread
    pub(crate) encoders: ThreadLocal<Mutex<Vec<CommandEncoder>>>,

    // Buffer staging pool
    pub(crate) staging: StagingPool,

    // ShaderC compiler
    pub(crate) shaderc: shaderc::Compiler,

    // Cached graphics data
    pub(crate) cached: Cached,
}

// Stats that can be displayed using egui
pub struct GraphicsStats {
    
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(crate) Arc<InternalGraphics>);

impl Graphics {
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

    // Get the global buffer allocator
    pub(crate) fn staging_pool(&self) -> &StagingPool {
        &self.0.staging
    }

    // Create a new command encoder to record commands
    // This might fetch an already existing command encoder (for this thread), or it will create a new one
    pub(crate) fn acquire(&self) -> CommandEncoder {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.pop().unwrap_or_else(|| {
            self.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            })
        })
    }

    // Submit one or multiple command encoders and possibly waits for the GPU to complete them
    // The submitted command encoders cannot be reused for new commands
    pub(crate) fn submit(
        &self,
        iter: impl IntoIterator<Item = CommandEncoder>,
        wait: bool
    ) {
        let finished = iter.into_iter().map(|x| x.finish());
        let i = self.queue().submit(finished);

        if wait {
            self.device().poll(Maintain::WaitForSubmissionIndex(i));
        }
    }

    // Submit all the currently unused command encoders and clears the thread local cache
    pub(crate) fn submit_unused(
        &self,
        wait: bool
    ) {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        self.submit(locked.drain(..), wait);
    }

    // Pushes some unfinished command encoders to be re-used by the current thread
    pub(crate) fn reuse(
        &self,
        iter: impl IntoIterator<Item = CommandEncoder>,
    ) {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.extend(iter);
    }
}
