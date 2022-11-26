use std::ffi::CString;

// Get the required validation layers
#[cfg(debug_assertions)]
pub fn required_validation_layers() -> Vec<CString> {
    vec![CString::new("VK_LAYER_KHRONOS_validation".to_owned()).unwrap()]
}

// No validation layers when we disable debug assertions
#[cfg(not(debug_assertions))]
pub fn required_get_validation_layers() -> Vec<CString> {
    vec![]
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
        ash::extensions::khr::Swapchain::name().to_owned()
    ]
}