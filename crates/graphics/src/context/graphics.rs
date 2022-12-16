use std::sync::Arc;
use vulkan::{Instance, Surface, Adapter, Device, Swapchain, Queue, Recorder, Submission};

// Internal struct that contain the raw vulkan instances and values
// This is what will be wrapped around an arc, and this is what will handle Vulkan object destruction
pub(super) struct InternalGraphics {
    pub(super) instance: Instance,
    pub(super) surface: Surface,
    pub(super) adapter: Adapter,
    pub(super) device: Device,
    pub(super) queue: Queue,
    pub(super) swapchain: Swapchain,
}

// Destroys the underlying Vulkan objects in proper
impl Drop for InternalGraphics {
    fn drop(&mut self) {
        unsafe {
            log::debug!("Dropping internal graphics handler...");

            log::debug!("Destroying swapchain...");
            self.swapchain.destroy(&self.device);

            log::debug!("Destroying queue...");
            self.queue.destroy(&self.device);

            log::debug!("Destroying surface...");
            self.surface.destroy();

            log::debug!("Destroying logical device...");
            self.device.destroy();

            log::debug!("Destroying Vulkan Instance...");
            self.instance.destroy();
            log::debug!("We did it guys, Vulkan is no more");
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

    // Get the adapter (physical device)
    pub fn adapter(&self) -> &Adapter {
        &self.0.adapter
    }

    // Get the device (logical device)
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

    // Acquire a new free command recorder that we can use to record commands
    pub fn acquire(&self) -> Recorder {
        unsafe { self.queue().acquire(self.device()) }
    }

    // Submit the command buffer and start executing the underlying commands
    pub fn submit(&self, recorder: Recorder) -> Submission {
        unsafe { self.queue().submit(self.device(), recorder) }
    }
}