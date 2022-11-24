use crate::Instance;
use ash::vk::{self};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

// Wrapper around the temporary surface loader
pub(crate) struct Surface {
    pub(crate) surface_loader: ash::extensions::khr::Surface,
    pub(crate) surface: vk::SurfaceKHR,
}

impl Surface {
    pub(crate) unsafe fn destroy(self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }
}

// Create the main Vulkan surface
pub(crate) unsafe fn create_surface(instance: &Instance) -> Surface {
    // Create a surface loader and the surface itself
    let surface = ash_window::create_surface(
        &instance.entry,
        &instance.instance,
        instance.raw_display_handle,
        instance.raw_window_handle,
        None,
    )
    .unwrap();
    let surface_loader = ash::extensions::khr::Surface::new(
        &instance.entry,
        &instance.instance,
    );

    Surface {
        surface_loader,
        surface,
    }
}
