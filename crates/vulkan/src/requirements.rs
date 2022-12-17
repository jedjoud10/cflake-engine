use std::ffi::CString;
use ash::vk::{
    self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
use vk::PhysicalDeviceType;
use crate::{AdapterFeatures, AdapterProperties, AdapterSurfaceProperties, AdapterQueueFamiliesProperties};

// Get the required validation layers
pub fn required_validation_layers() -> Vec<CString> {
    #[cfg(debug_assertions)]
    return vec![CString::new("VK_LAYER_KHRONOS_validation".to_owned())
        .unwrap()];

    #[cfg(not(debug_assertions))]
    return vec![];
}

// Get the required instance extensions
pub fn required_instance_extensions() -> Vec<CString> {
    vec![
        ash::extensions::ext::DebugUtils::name().to_owned(),
        ash::extensions::khr::Surface::name().to_owned(),
    ]
}
    
// Get the reqwuired logical device extensions
pub fn required_device_extensions() -> Vec<CString> {
    vec![
        ash::extensions::khr::Swapchain::name().to_owned(),
        ash::extensions::khr::Synchronization2::name().to_owned(),
    ]
}

// Get the features that we will use for the device
pub fn required_features() -> AdapterFeatures {
    let features = *vk::PhysicalDeviceFeatures::builder()
        .tessellation_shader(true)
        .multi_draw_indirect(true)
        .draw_indirect_first_instance(true)
        .sampler_anisotropy(true)
        .shader_float64(true)
        .robust_buffer_access(true)
        .shader_int64(true);

    let features11 = *vk::PhysicalDeviceVulkan11Features::builder();

    let features12 = *vk::PhysicalDeviceVulkan12Features::builder();

    let features13 = *vk::PhysicalDeviceVulkan13Features::builder()
        .robust_image_access(true)
        .synchronization2(true);

    AdapterFeatures { features, features11, features12, features13 }
}

// The required Vulkan API version
pub fn required_api_version() -> u32 {
    vk::API_VERSION_1_3
}