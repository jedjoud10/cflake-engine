use std::sync::Arc;
use vulkan::{Instance, Surface, Adapter, Device, Swapchain, Queue, Recorder, Submission};

// Internal struct that contain the raw vulkan instances and values
// This is what will be wrapped around an arc, and this is what will handle Vulkan object destruction
pub(super) struct InternalGraphics {
    pub instance: Instance,
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub swapchain: Swapchain,
}

// Destroys the underlying Vulkan objects in proper
impl Drop for InternalGraphics {
    fn drop(&mut self) {
        unsafe {
            log::warn!("Dropping internal graphics handler...");

            log::warn!("Destroying swapchain...");
            self.swapchain.destroy(&self.device);

            log::warn!("Destroying queue...");
            self.queue.destroy(&self.device);

            log::warn!("Destroying surface...");
            self.surface.destroy();

            log::warn!("Destroying logical device...");
            self.device.destroy();

            log::warn!("Destroying Vulkan Instance...");
            self.instance.destroy();
            log::warn!("We did it guys, Vulkan is no more");
        }
    }
}

// Graphical context that we will wrap around the Vulkan instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(super) Arc<InternalGraphics>);

impl Graphics {
    // Get the instance
    pub fn instance(&self) -> &Instance {
        &self.0.instance
    }

    // Get the adapter
    pub fn adapter(&self) -> &Adapter {
        &self.0.adapter
    }

    // Get the device
    pub fn device(&self) -> &Device {
        &self.0.device
    }

    // Get the main queues
    pub fn queue(&self) -> &Queue {
        &self.0.queue
    }

    // Get the surface
    pub fn surface(&self) -> &Surface {
        &self.0.surface
    }

    // Get the swapchain
    pub fn swapchain(&self) -> &Swapchain {
        &self.0.swapchain
    }

    // Aquire a new free command recorder that we can use to record commands
    pub fn acquire(&self) -> Recorder {
        unsafe { self.queue().acquire(self.device()) }
    }

    // Submit the command buffer and start executing the underlying commands
    pub fn submit(&self, recorder: Recorder) -> Submission {
        unsafe { self.queue().submit(self.device(), recorder) }
    }
}