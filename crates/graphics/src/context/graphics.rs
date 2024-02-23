use ahash::AHashMap;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::{hash::BuildHasherDefault, path::PathBuf, sync::Arc, cell::RefCell};
use thread_local::ThreadLocal;
use utils::Storage;
pub use wgpu::CommandEncoder;
use wgpu::{
    util::StagingBelt, Adapter, Device, Instance, Maintain, Queue, Sampler, Surface,
    SurfaceCapabilities, SurfaceConfiguration, TextureView,
};

thread_local! {
    static FOO: RefCell<Vec<CommandEncoder>> = RefCell::default();
}

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) adapter: Adapter,
    pub(crate) queue: Queue,
}

/// Stats that can be displayed using egui
#[derive(Default, Clone, Copy)]
pub struct GraphicsStats {
    /// Max number of adapters supported
    pub adapters: usize,

    /// Max number of devices created
    pub devices: usize,

    /// Number of pipeline layouts created
    pub pipeline_layouts: usize,

    /// Number of shader modules created
    pub shader_modules: usize,

    /// Number of bind group layouts created
    pub bind_group_layouts: usize,

    /// Number of bind groups created
    pub bind_groups: usize,

    /// Number of active command buffers
    pub command_buffers: usize,

    /// Number of render pipelines created
    pub render_pipelines: usize,

    /// Number of compute pipelines created
    pub compute_pipelines: usize,
    
    /// Number of current wgpu buffers
    pub buffers: usize,

    /// Number of current wgpu textures
    pub textures: usize,

    /// Number of current wgpu texture views
    pub texture_views: usize,

    /// Number of current wgpu samplers
    pub samplers: usize,
}

/// Graphical context that we will wrap around the WGPU instance
/// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(crate) Arc<InternalGraphics>);

impl Graphics {
    /// Get the internally stored instance
    pub fn instance(&self) -> &Instance {
        &self.0.instance
    }

    /// Get the internally stored device
    pub fn device(&self) -> &Device {
        &self.0.device
    }

    /// Get the internally stored queue
    pub fn queue(&self) -> &Queue {
        &self.0.queue
    }

    /// Get the GPU we are using
    pub fn adapter(&self) -> &Adapter {
        &self.0.adapter
    }

    /// Create a new command encoder to record commands
    /// This might fetch an already existing command encoder (for this thread), or it will create a new one
    pub fn acquire(&self) -> CommandEncoder {
        FOO.with_borrow_mut(|locked|         locked.pop().unwrap_or_else(|| {
            self.device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
        }))

        /*
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.pop().unwrap_or_else(|| {
            self.device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
        })
        */
    }

    /// Submit one or multiple command encoders and possibly waits for the GPU to complete them
    /// The submitted command encoders cannot be reused for new commands
    pub fn submit_from_iter(&self, iter: impl IntoIterator<Item = CommandEncoder>, wait: bool) {
        let finished = iter.into_iter().map(|x| x.finish());
        let i = self.queue().submit(finished);

        if wait {
            self.device().poll(Maintain::WaitForSubmissionIndex(i));
        }
    }

    /// Submit all the currently unused command encoders and clears the thread local cache
    pub fn submit(&self, wait: bool) {
        FOO.with_borrow_mut(|locked| {
            self.submit_from_iter(locked.drain(..), wait)
        });
        
        /*
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        self.submit_from_iter(drained, wait);
        */
    }

    /// Pushes some unfinished command encoders to be re-used by the current thread
    pub fn reuse(&self, iter: impl IntoIterator<Item = CommandEncoder>) {
        FOO.with_borrow_mut(|locked| locked.extend(iter));
        /*
        let encoders = self.0.encoders.get_or_default();
        let mut locked = encoders.lock();
        locked.extend(iter);
        */
    }
}
