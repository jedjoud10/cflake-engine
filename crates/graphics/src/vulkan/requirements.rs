use super::AdapterFeatures;
use ash::vk::{self};
use std::ffi::CString;

// Get the required validation layers
pub fn required_validation_layers() -> Vec<CString> {
    #[cfg(debug_assertions)]
    return vec![CString::new(
        "VK_LAYER_KHRONOS_validation".to_owned(),
    )
    .unwrap()];

    #[cfg(not(debug_assertions))]
    return Vec::new();
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
        ash::extensions::khr::DynamicRendering::name().to_owned(),
        ash::extensions::khr::BufferDeviceAddress::name().to_owned(),
        ash::extensions::khr::PushDescriptor::name().to_owned(),
    ]
}

// The required Vulkan API version
pub fn required_api_version() -> u32 {
    vk::API_VERSION_1_3
}
