use ash::{
    extensions::ext::DebugUtils,
    vk::{self},
    Entry,
};

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle,
    RawWindowHandle,
};
use std::{
    borrow::Cow,
    ffi::{c_void, CStr, CString},
};

// Wrapper around Vulkan entry and Vulkan instance
pub struct Instance {
    // Context related
    pub entry: Entry,
    pub instance: ash::Instance,
    pub(crate) raw_display_handle: RawDisplayHandle,
    pub(crate) raw_window_handle: RawWindowHandle,

    // Only enable validation and message logging in debug mode
    #[cfg(debug_assertions)]
    debug_utils: DebugUtils,
    #[cfg(debug_assertions)]
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

// This instance is shareable
unsafe impl Sync for Instance {}
unsafe impl Send for Instance {}

impl Instance {
    // Create the main Vulkan instance
    pub unsafe fn new(
        window: &winit::window::Window,
        instance_extensions: Vec<CString>,
        validation_layers: Vec<CString>,
        app_title: String,
        engine_title: String,
    ) -> Instance {
        // Load the loading functions
        let entry = Entry::load().unwrap();

        // Get a window and display handle to the winit window
        let raw_display_handle = window.raw_display_handle();
        let raw_window_handle = window.raw_window_handle();

        // Create the app info
        let app_name = CString::new(app_title.clone()).unwrap();
        let engine_name = CString::new(engine_title).unwrap();
        let app_info = *vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(vk::API_VERSION_1_3)
            .application_version(0)
            .engine_version(0)
            .engine_name(&engine_name);

        // Create the debug messenger create info
        #[cfg(debug_assertions)]
        let mut debug_messenger_create_info =
            super::create_debug_messenger_create_info();

        // Get the required instance extensions from the handle
        let mut extension_names_ptrs =
            ash_window::enumerate_required_extensions(
                raw_display_handle,
            )
            .unwrap()
            .to_vec();
        extension_names_ptrs
            .extend(instance_extensions.iter().map(|s| s.as_ptr()));

        // Get the required validation layers
        let validation_ptrs = validation_layers
            .iter()
            .map(|cstr| cstr.as_ptr())
            .collect::<Vec<_>>();

        // Setup the instance create info (with debug info)
        #[cfg(debug_assertions)]
        let instance_create_info = *vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&validation_ptrs)
            .enabled_extension_names(&extension_names_ptrs)
            .push_next(&mut debug_messenger_create_info);

        // Setup the instance create info (without debug info)
        #[cfg(not(debug_assertions))]
        let instance_create_info = *vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&validation_ptrs)
            .enabled_extension_names(&extension_names_ptrs);

        // Create the instance
        let instance = entry
            .create_instance(&instance_create_info, None)
            .unwrap();

        // Create the debug utils
        #[cfg(debug_assertions)]
        let debug_utils = DebugUtils::new(&entry, &instance);

        // Create the debug messenger
        #[cfg(debug_assertions)]
        let debug_messenger = debug_utils
            .create_debug_utils_messenger(
                &debug_messenger_create_info,
                None,
            )
            .unwrap();

        Instance {
            entry,
            instance,

            #[cfg(debug_assertions)]
            debug_utils,

            #[cfg(debug_assertions)]
            debug_messenger,

            raw_display_handle,
            raw_window_handle,
        }
    }

    // Destroy the instance
    pub unsafe fn destroy(self) {
        #[cfg(debug_assertions)]
        self.debug_utils.destroy_debug_utils_messenger(
            self.debug_messenger,
            None,
        );

        self.instance.destroy_instance(None);
    }
}
