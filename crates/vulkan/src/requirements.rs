use std::ffi::CString;
use ash::vk::{
    self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
use vk::PhysicalDeviceType;

use crate::{AdapterFeatures, AdapterProperties, AdapterSurfaceProperties, AdapterQueueFamiliesProperties};

// Check wether or not a physical device is suitable for rendering
// This checks the minimum requirements that we need to achieve to be able to render
pub(super) fn is_physical_device_suitable(
    features: &AdapterFeatures,
    properties: &AdapterProperties,
    surface: &AdapterSurfaceProperties,
    families: &AdapterQueueFamiliesProperties,
) -> bool {
    log::debug!(
        "Checking if adapter {} is suitable...",
        properties.name
    );

    // Check all the requirements that are needed for us to use this Adapter
    let double_buffering_supported =
        is_double_buffering_supported(surface.surface_capabilities);
    let format_supported = is_surface_format_supported(&surface.present_formats);
    //let features_supported = is_feature_list_supported(features);
    let present_supported = is_present_mode_supported(&surface.present_modes);
    let device_type_okay = is_device_type_optimal(properties.device_type);

    // All the checks must pass
    double_buffering_supported
        && format_supported
        && present_supported
        && device_type_okay
}

// Check if the Adapter is optimal (dGPU)
fn is_device_type_optimal(_type: PhysicalDeviceType) -> bool {
    let device_type_okay = _type == PhysicalDeviceType::DISCRETE_GPU;
    log::debug!("Adapter Device Type: {:?}", _type);
    device_type_okay
}


// Check if the Adapter supports a min image count of 2
fn is_double_buffering_supported(
    surface: SurfaceCapabilitiesKHR,
) -> bool {
    let double_buffering_supported = surface.min_image_count == 2;
    log::debug!(
        "Adapter Double Buffering: {}",
        double_buffering_supported
    );
    double_buffering_supported
}

// Check if the Adapter present modes support FIFO_RELAXED and IMMEDIATE
fn is_present_mode_supported(modes: &[PresentModeKHR]) -> bool {
    let present_supported = modes
        .iter()
        .find(|&&present| {
            let relaxed = present == vk::PresentModeKHR::FIFO_RELAXED;
            let immediate = present == vk::PresentModeKHR::IMMEDIATE;
            relaxed || immediate
        })
        .is_some();

    present_supported
}

// Check if the Adapter formats supportB8G8R8A8_SRGB and SRGB_NONLINEAR
fn is_surface_format_supported(formats: &[SurfaceFormatKHR]) -> bool {
    let format_supported = formats
        .iter()
        .find(|format| {
            let format_ = format.format == vk::Format::B8G8R8A8_SRGB;
            let color_space_ = format.color_space
                == vk::ColorSpaceKHR::SRGB_NONLINEAR;
            format_ && color_space_
        })
        .is_some();
    log::debug!(
        "Adapter Swapchain Format Supported: {}",
        format_supported
    );
    format_supported
}

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
        .shader_int64(true);

    let features11 = *vk::PhysicalDeviceVulkan11Features::builder();

    let features12 = *vk::PhysicalDeviceVulkan12Features::builder();

    let features13 = *vk::PhysicalDeviceVulkan13Features::builder()
        .synchronization2(true);

    AdapterFeatures { features, features11, features12, features13 }
}

// The required Vulkan API version
pub fn required_api_version() -> u32 {
    vk::API_VERSION_1_3
}