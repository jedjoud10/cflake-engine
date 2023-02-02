use ash::{extensions::khr, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

use super::Instance;

// This is a surface that matches up with the Vulkan surface KHR extension
pub struct Surface {
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Surface {
    // Create a new surface from the instance (assumes that the extension was already set)
    pub unsafe fn new(
        instance: &Instance,
        window: &Window,
    ) -> Surface {
        // Create a surface loader and the surface itself
        let surface = ash_window::create_surface(
            instance.entry(),
            instance.raw(),
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
        .unwrap();
        log::debug!("Created the Vulkan surface successfully");
        let surface_loader =
            khr::Surface::new(instance.entry(), instance.raw());
        log::debug!("Created the Vulkan surface loader successfully");
        Surface {
            surface_loader,
            surface,
        }
    }

    // Get the underlying raw surface loader
    pub fn surface_loader(&self) -> &khr::Surface {
        &self.surface_loader
    }

    // Get the underlying raw surface
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    // Destroy the surface
    pub unsafe fn destroy(&self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }
}
