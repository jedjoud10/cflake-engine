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

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,
    pub(crate) encoders: ThreadLocal<Mutex<Vec<CommandEncoder>>>,
}

// Stats that can be displayed using egui
#[derive(Default, Clone, Copy)]
pub struct GraphicsStats {
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

    // Create a new command encoder to record commands
    // This might fetch an already existing command encoder (for this thread), or it will create a new one
    pub fn acquire(&self) -> CommandEncoder {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.pop().unwrap_or_else(|| {
            self.device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
        })
    }

    // Submit one or multiple command encoders and possibly waits for the GPU to complete them
    // The submitted command encoders cannot be reused for new commands
    pub fn submit_from_iter(&self, iter: impl IntoIterator<Item = CommandEncoder>, wait: bool) {
        let finished = iter.into_iter().map(|x| x.finish());
        let i = self.queue().submit(finished);

        if wait {
            self.device().poll(Maintain::WaitForSubmissionIndex(i));
        }
    }

    // Submit all the currently unused command encoders and clears the thread local cache
    pub fn submit(&self, wait: bool) {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        self.submit_from_iter(locked.drain(..), wait);
    }

    // Pushes some unfinished command encoders to be re-used by the current thread
    pub fn reuse(&self, iter: impl IntoIterator<Item = CommandEncoder>) {
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.extend(iter);
    }
}
