use std::sync::Arc;

use crate::{Instance, Surface, Adapter, Device, Swapchain, Queue, Recorder, Submission};

// Internal struct that contain the raw vulkan instances and values
// This is what will be wrapped around an arc, and this is what will handle Vulkan object destruction
struct InternalGraphics {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swapchain: Swapchain,
}

// Destroys the underlying Vulkan objects in proper
impl Drop for InternalGraphics {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.destroy(&self.device);
            self.queue.destroy(&self.device);
            self.surface.destroy();
            self.device.destroy();
            self.instance.destroy();
        }
    }
}

// Graphical context that we will wrap around the Vulkan instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(Arc<InternalGraphics>);

impl Graphics {
    // Create a new graphics wrapper from the raw Vulkan wrappers
    pub(crate) fn new(
        instance: Instance,
        surface: Surface,
        adapter: Adapter,
        device: Device,
        queue: Queue,
        swapchain: Swapchain,
    ) -> Self {
        Self(Arc::new(InternalGraphics {
            instance,
            surface,
            adapter,
            device,
            queue,
            swapchain,
        }))
    }

    // Get the instance
    pub(crate) fn instance(&self) -> &Instance {
        &self.0.instance
    }

    // Get the adapter
    pub(crate) fn adapter(&self) -> &Adapter {
        &self.0.adapter
    }

    // Get the device
    pub(crate) fn device(&self) -> &Device {
        &self.0.device
    }

    // Get the main queues
    pub(crate) fn queue(&self) -> &Queue {
        &self.0.queue
    }

    // Get the surface
    pub(crate) fn surface(&self) -> &Surface {
        &self.0.surface
    }

    // Get the swapchain
    pub(crate)fn swapchain(&self) -> &Swapchain {
        &self.0.swapchain
    }

    // Aquire a new free command recorder that we can use to record commands
    pub fn acquire(&self) -> Recorder {
        unsafe { self.queue().acquire(self.device(), false) }
    }

    // Submit the command buffer and start executing the underlying commands
    pub fn submit(&self, recorder: Recorder) -> Submission {
        unsafe { self.queue().submit(self.device(), recorder) }
    }
}