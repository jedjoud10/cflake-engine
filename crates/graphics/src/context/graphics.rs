use std::ffi::{CStr, CString};

use super::Window;
use ash::{extensions::{ext::DebugUtils, khr::Swapchain}, vk, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicSettings {
    pub validation_layers: Vec<CString>,
    pub instance_extensions: Vec<CString>,
    pub logical_device_extensions: Vec<CString>,
}

impl Default for GraphicSettings {
    fn default() -> Self {
        Self {
            validation_layers: vec![CString::new(
                "VK_LAYER_KHRONOS_validation".to_owned(),
            )
            .unwrap()],
            instance_extensions: vec![DebugUtils::name().to_owned()],
            logical_device_extensions: vec![Swapchain::name().to_owned()]
        }
    }
}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
pub struct Graphics {
    entry: Entry,
    instance: Instance,
    debug_utils: DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Graphics {
    // Create a new Vulkan graphics context based on the window wrapper
    pub(crate) unsafe fn new(
        title: &str,
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
    ) -> Graphics {
        // Load the loading functions
        let entry = Entry::load().unwrap();
        let version =
            entry.try_enumerate_instance_version().unwrap().unwrap();

        // Get a window and display handle to the winit window
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();

        // Create the app info
        let app_name = CString::new(title.to_owned()).unwrap();
        let engine_name = CString::new("cFlake engine").unwrap();
        let app_info = *vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(vk::API_VERSION_1_3)
            .application_version(0)
            .engine_version(0)
            .engine_name(&engine_name);

        // Create the debug messenger create info
        let mut debug_messenger_create_info = *vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(super::debug_callback));

        // Get the required instance extensions from the handle
        let mut extension_names_ptrs =
            ash_window::enumerate_required_extensions(display_handle)
                .unwrap()
                .to_vec();
        extension_names_ptrs.extend(
            graphic_settings
                .instance_extensions
                .iter()
                .map(|s| s.as_ptr()),
        );

        // Get the required validation layers
        let validation_ptrs = graphic_settings
            .validation_layers
            .iter()
            .map(|cstr| cstr.as_ptr())
            .collect::<Vec<_>>();

        // Setup the instance create info
        let instance_create_info = *vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&validation_ptrs)
            .enabled_extension_names(&extension_names_ptrs)
            .push_next(&mut debug_messenger_create_info);

        // Create the instance
        let instance = entry
            .create_instance(&instance_create_info, None)
            .unwrap();

        // Create the debug messenger and the debug utils
        let debug_utils = DebugUtils::new(&entry, &instance);
        let debug_messenger = debug_utils
            .create_debug_utils_messenger(
                &debug_messenger_create_info,
                None,
            )
            .unwrap();

        Self {
            entry,
            instance,
            debug_utils,
            debug_messenger,
        }
    }

    // Get access to the internal raw instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    // Get access to the internal raw entry
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    // Destroy the context after we've done using it
    pub(crate) unsafe fn destroy(self) {
        self.debug_utils.destroy_debug_utils_messenger(
            self.debug_messenger,
            None,
        );
        self.instance.destroy_instance(None);
    }
}
