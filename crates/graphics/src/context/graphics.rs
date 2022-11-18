use std::ffi::CString;

use super::Window;
use ash::{Instance, extensions::ext::DebugUtils, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicsSettings {
    pub validation_layers: Vec<CString>,
    pub instance_extensions: Vec<CString>,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            validation_layers: vec![CString::new("VK_LAYER_KHRONOS_validation".to_owned()).unwrap()],
            instance_extensions: vec![DebugUtils::name().to_owned()]
        }
    }
}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
pub struct Graphics {    
    instance: Instance,
    debug_utils: DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    physical: vk::PhysicalDevice,
    logical: vk::Device,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl Graphics {
    // Create a new Vulkan graphics context based on the window wrapper
    pub(crate) unsafe fn new(window: &Window, settings: GraphicsSettings) -> Graphics {
        // Load the loading functions
        let entry = ash::Entry::load().unwrap();
        let version = entry.try_enumerate_instance_version().unwrap().unwrap();

        // Get a window and display handle to the winit window
        let display_handle = window.raw().raw_display_handle();
        let window_handle = window.raw().raw_window_handle();

        // Create the app info
        let app_info = create_app_info(window);

        // Create a vulkan instance builder
        let mut debug_messenger_create_info = create_debug_messenger_create_info();
        let instance_create_info = create_instance_create_info(settings, display_handle, &app_info, &mut debug_messenger_create_info);

        // Create the instace
        let instance = create_instance(instance_create_info, &entry);

        let debug_messenger = create_debug_messenger(&entry, &instance, *debug_messenger_create_info);

        // Create a surface loader
        create_khr_surface(&entry, &instance, display_handle, window_handle);

        todo!()
    }

    // Destroy the context after we've done using it
    pub(crate) unsafe fn destroy(self) {
        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
        self.instance.destroy_instance(None);
    }
}

// Create the app info builder
fn create_app_info(window: &Window) -> vk::ApplicationInfo {
    let app_name = CString::new(window.settings().title.clone()).unwrap();
    let engine_name = CString::new("cFlake engine").unwrap();
    let app_info = *vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .api_version(vk::API_VERSION_1_3)
        .application_version(0)
        .engine_version(0)
        .engine_name(&engine_name);
    app_info
}

// Create the KHR surface
unsafe fn create_khr_surface(entry: &ash::Entry, instance: &Instance, display_handle: raw_window_handle::RawDisplayHandle, window_handle: raw_window_handle::RawWindowHandle) {
    let surface = ash_window::create_surface(
        entry,
        instance,
        display_handle,
        window_handle,
        None,
    )
    .unwrap();
    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
}

// Create the instance create info 
unsafe fn create_instance_create_info<'a>(settings: GraphicsSettings, display_handle: raw_window_handle::RawDisplayHandle, app_info: &vk::ApplicationInfo, debug_messenger_create_info: &mut vk::DebugUtilsMessengerCreateInfoEXT) -> vk::InstanceCreateInfo {
    // Get the required instance extensions from the handle
    let mut extension_names_ptrs =
        ash_window::enumerate_required_extensions(display_handle)
            .unwrap()
            .to_vec();
    extension_names_ptrs.extend(settings.instance_extensions.iter().map(|s| s.as_ptr()));
    
    // Get the required validation layers
    let validation_ptrs = settings.validation_layers.iter().map(|cstr| cstr.as_ptr()).collect::<Vec<_>>();
    
    // Setup the instance create info
    *vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&validation_ptrs)
        .enabled_extension_names(&extension_names_ptrs)
        .push_next(debug_messenger_create_info)
}

// Create the instance
unsafe fn create_instance(create_info: vk::InstanceCreateInfo, entry: &ash::Entry) -> Instance {
    entry.create_instance(&create_info, None).unwrap()
}

// Create the debug messenger create info
unsafe fn create_debug_messenger_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'a> {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(super::debug_callback))
}

// Create the debug messenger
unsafe fn create_debug_messenger(entry: &ash::Entry, instance: &Instance, debug_messenger_create_info: vk::DebugUtilsMessengerCreateInfoEXT) -> vk::DebugUtilsMessengerEXT {
    let debug_utils = DebugUtils::new(&entry, &instance);
    debug_utils
        .create_debug_utils_messenger(&debug_messenger_create_info, None)
        .unwrap()
}