use std::sync::Arc;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use vulkan::{
    Adapter, Device, Instance, Queue, Recorder, Submission, Surface,
    Swapchain,
};

// We have a global graphical context
// TODO: Find alternate of OnceCell and lazy_Static
lazy_static::lazy_static! {
    pub(crate) static ref CONTEXT: RwLock<Option<Graphics>> = RwLock::new(None);
}

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
    pub fn global() -> parking_lot::MappedRwLockReadGuard<'static, Self> {
        let initialized = CONTEXT.read();
        parking_lot::RwLockReadGuard::map(initialized, |x| x.as_ref().unwrap())
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

    // Destroy the graphical context
    pub(crate) unsafe fn destroy(self) {
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
