use dashmap::DashMap;
use naga::{front::glsl::Parser, valid::Validator};
use parking_lot::Mutex;
use std::{hash::BuildHasherDefault, sync::Arc};
use utils::Storage;
use wgpu::{
    util::StagingBelt, Adapter, CommandEncoder, Device, Queue,
    Sampler, Surface, SurfaceCapabilities, SurfaceConfiguration,
    TextureView,
};

use crate::{
    BindGroupLayout, ReflectedShader, SamplerSettings, SamplerWrap,
    StagingPool,
};

type NoHash = BuildHasherDefault<nohash_hasher::NoHashHasher<u64>>;

// Cached graphics data
pub(crate) struct Cached {
    pub(crate) samplers: DashMap<SamplerSettings, Arc<Sampler>>,
    pub(crate) bind_group_layouts:
        DashMap<BindGroupLayout, Arc<wgpu::BindGroupLayout>>,
    pub(crate) pipeline_layouts:
        DashMap<ReflectedShader, Arc<wgpu::PipelineLayout>>,
    pub(crate) bind_groups:
        DashMap<Vec<wgpu::Id>, Arc<wgpu::BindGroup>>,
}

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    // Device and queue
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,

    // Buffer staging pool
    pub(crate) staging: StagingPool,

    // ShaderC compiler
    pub(crate) shaderc: shaderc::Compiler,

    // Cached graphics data
    pub(crate) cached: Cached,
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
    pub fn staging_pool(&self) -> &StagingPool {
        &self.0.staging
    }

    // Create a new command list to record commands
    pub fn acquire(&self) -> CommandEncoder {
        self.device().create_command_encoder(&Default::default())
    }

    // Submit one or multiple command lists and return a fence
    pub fn submit(
        &self,
        encoders: impl IntoIterator<Item = CommandEncoder>,
    ) {
        let finished = encoders.into_iter().map(|x| x.finish());
        self.queue().submit(finished);
    }
}
