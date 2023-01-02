use std::sync::Arc;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use vulkan::{
    Adapter, Device, Instance, Queue, Recorder, Submission, Surface,
    Swapchain,
};

/*
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
*/

// We have a global graphical context
pub(crate) static CONTEXT: OnceCell<Graphics> = OnceCell::new();

// Graphical context that we will wrap around the Vulkan instance
// This context must be shareable between threads to allow for multithreading
pub struct Graphics {
    pub(super) instance: Instance,
    pub(super) surface: Surface,
    pub(super) adapter: Adapter,
    pub(super) device: Device,
    pub(super) queue: Queue,
    pub(super) swapchain: Swapchain,
}

impl Graphics {
    // Get the graphics from the global graphical context
    // Panics if the graphics context wasn't created yet
    pub fn global() -> &'static Self {
        CONTEXT.get().unwrap()
    }

    // Get the instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    // Get the adapter (physical device)
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    // Get the device (logical device)
    pub fn device(&self) -> &Device {
        &self.device
    }

    // Get the main queues
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    // Get the surface
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    // Get the swapchain
    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }
}
