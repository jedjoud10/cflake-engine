use std::ffi::CString;

use super::Window;
use ash::{Instance, extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawDisplayHandle;
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicsSettings {
    pub validation_layers: Vec<String>,
}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
pub struct Graphics {    
    instance: Instance,
    debug_utils: DebugUtils,
    physical: vk::PhysicalDevice,
    logical: vk::Device,
    surface: vk::SurfaceKHR,
}

impl Graphics {
    // Create a new Vulkan graphics context based on the window wrapper
    pub(crate) unsafe fn new(window: &Window, settings: GraphicsSettings) -> Graphics {
        // Load the loading functions
        let entry = ash::Entry::load().unwrap();
        let version = entry.try_enumerate_instance_version().unwrap().unwrap();

        // Create the app info
        let app_name = CString::new(window.settings().title.clone()).unwrap();
        let engine_name = CString::new("cFlake engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(vk::API_VERSION_1_3)
            .application_version(0)
            .engine_version(0)
            .engine_name(&engine_name);

        // Get a window handle to the winit window
        let handle = window.raw().raw_display_handle();
            
            // Get the required extensions from the handle
        let mut extension_names =
            ash_window::enumerate_required_extensions(handle)
                .unwrap()
                .to_vec();
        extension_names.push(DebugUtils::name().as_ptr());

        // Get the required validation layers
        let validation = settings.validation_layers.into_iter().map(CString::new).filter_map(|res| res.ok()).collect::<Vec<_>>();
        let validation_ptrs = validation.iter().map(|cstr| cstr.as_ptr()).collect::<Vec<_>>();

        todo!()
    }

    // Destroy the context after we've done using it
    pub(crate) fn destroy(self) {
        
    }
}
