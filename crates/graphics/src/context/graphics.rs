use dashmap::DashMap;
use naga::{front::glsl::Parser, valid::Validator};
use parking_lot::Mutex;
use utils::Storage;
use std::sync::Arc;
use wgpu::{
    util::StagingBelt, Device, Queue, Surface, SurfaceCapabilities,
    SurfaceConfiguration, TextureView, Sampler, Adapter,
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
    pub(crate) samplers: Mutex<Vec<(u64, Arc<Sampler>)>>,

    // Shader compiler and validator
    pub(crate) parser: Mutex<Parser>,
    pub(crate) validator: Mutex<Validator>,
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

    // Get the GLSL shader parser
    // TODO: Make this thread local
    pub fn parser(&self) -> &Mutex<Parser> {
        &self.0.parser
    }

    // Get the Naga shader validator
    // TODO: Make this thread local
    pub fn validator(&self) -> &Mutex<Validator> {
        &self.0.validator
    }

    // Get the wgpu buffer staging belt
    pub fn staging_belt(&self) -> &Mutex<StagingBelt> {
        &self.0.staging
    }
}
