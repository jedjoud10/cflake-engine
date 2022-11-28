use crate::Instance;
use ash::{
    extensions::khr,
    vk::{self},
};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

// Wrapper around the temporary surface loader
pub struct Surface {
    pub(super) surface_loader: khr::Surface,
    pub(super) surface: vk::SurfaceKHR,
}

impl Surface {
    // Create the main Vulkan surface
    pub unsafe fn new(instance: &Instance) -> Surface {
        // Create a surface loader and the surface itself
        let surface = ash_window::create_surface(
            &instance.entry,
            &instance.instance,
            instance.raw_display_handle,
            instance.raw_window_handle,
            None,
        )
        .unwrap();
        let surface_loader =
            khr::Surface::new(&instance.entry, &instance.instance);

        Surface {
            surface_loader,
            surface,
        }
    }

    // Destroy the surface and the loader
    pub unsafe fn destroy(self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }

    // Get the internal surface loader
    pub fn surface_loader(&self) -> &khr::Surface {
        &self.surface_loader
    }

    // Get the internal surface
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }
}
