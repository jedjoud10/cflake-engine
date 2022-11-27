use crate::FrameRateLimit;

use super::WindowSettings;
use std::{ffi::CString, sync::Arc};
use vulkan::*;

// Internal graphics context that will be shared with other threads
pub(crate) struct InternalGraphics {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queues: Queues,
    surface: Surface,
    swapchain: Swapchain,
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(Arc<InternalGraphics>);

impl Graphics {
    // Create a new Vulkan graphics context based on the window wrapper
    // This will create the window surface, then pick out a physical device
    // It will then create the swapchain and setup the swapchain images
    pub(crate) unsafe fn new(
        window: &winit::window::Window,
        window_settings: &WindowSettings,
    ) -> Graphics {
        let validation_layers = vulkan::required_validation_layers();
        let instance_extensions = vulkan::required_instance_extensions();
        let device_extensions = vulkan::required_device_extensions();

        // Create the Vulkan entry and instance
        let instance = Instance::new(
            window,
            instance_extensions,
            validation_layers,
            window_settings.title.clone(),
            "cFlake Engine".to_owned()
        );

        // Create a surface from the KHR extension
        let surface = Surface::new(&instance);

        // Pick a physical device (adapter)
        let adapter = Adapter::pick(
            &instance,
            false,
            &surface,
        );

        // Create the queues that we will instantiate
        let mut queues = Queues::new(&adapter, &instance);

        // Create a new device with those queues
        let device = Device::new(
            &instance,
            &adapter,
            &mut queues,
            device_extensions,
        );

        // Create a swapchain we can render to
        let vsync = matches!(window_settings.limit, FrameRateLimit::VSync);
        let swapchain = Swapchain::new(
            &adapter,
            &surface,
            &device,
            &instance,
            window,
            vsync,
        );

        Self(Arc::new(InternalGraphics {
            instance,
            adapter,
            device,
            queues,
            surface,
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

    // Get the queues
    pub(crate) fn queues(&self) -> &Queues {
        &self.0.queues
    }

    // Get the surface
    pub(crate) fn surface(&self) -> &Surface {
        &self.0.surface
    }

    // Get the swapchain
    pub(crate) fn swapchain(&self) -> &Swapchain {
        &self.0.swapchain
    }

    // Draw the main window swapchain sheize
    pub(crate) unsafe fn draw(&mut self, _value: f32) {
        // Get the next free image
        let image = self.swapchain().aquire(); 
        self.swapchain().render(self.device(), self.queues(), image);
        self.swapchain().present(self.device(),self.queues(), image);
    }

    // Destroy the context after we've done using it
    // Only destroy the context when we are sure we have no shared state
    pub(crate) unsafe fn destroy(self) {
        /*
        internal.swapchain.destroy(&internal.device);
        internal.queues.destroy(&internal.device);
        internal.device.destroy();
        internal.surface.destroy();
        internal.instance.destroy();
        */
    }
}