use std::sync::Arc;

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    // Device and queue
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) staging: wgpu::util::StagingBelt,

    // Surface related
    pub(crate) window: Arc<winit::window::Window>,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) surface_capabilities: wgpu::SurfaceCapabilities,
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(crate) Arc<InternalGraphics>);

impl Graphics {
    // Get the internally stored device
    pub fn device(&self) -> &wgpu::Device {
        &self.0.device
    }

    // Get the internally stored queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.0.queue
    }
    
    // Get the internally stored surface 
    pub fn surface(&self) -> &wgpu::Surface {
        &self.0.surface
    }
}
