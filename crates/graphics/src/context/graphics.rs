use dashmap::DashMap;
use naga::{front::glsl::Parser, valid::Validator};
use parking_lot::Mutex;
use utils::Storage;
use std::sync::Arc;
use wgpu::{
    util::StagingBelt, Device, Queue, Surface, SurfaceCapabilities,
    SurfaceConfiguration, TextureView, Sampler, Adapter, CommandEncoder,
};

use crate::{SamplerWrap, SamplerSettings};

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    // Device and queue
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,

    // Buffer staging belt
    pub(crate) staging: Mutex<StagingBelt>,

    // Cached texture samplers 
    pub(crate) samplers: DashMap<SamplerSettings, Arc<Sampler>>,
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

    // Get the wgpu buffer staging belt
    pub fn staging_belt(&self) -> &Mutex<StagingBelt> {
        &self.0.staging
    }

    // Create a new command list to record commands
    pub fn acquire(&self) -> CommandEncoder {
        self.device().create_command_encoder(&Default::default())
    }
    
    // Submit one or multiple command lists and return a fence
    pub fn submit(&self, encoders: impl IntoIterator<Item = CommandEncoder>) {
        let finished = encoders.into_iter().map(|x| x.finish());
        self.queue().submit(finished);
    }
}
