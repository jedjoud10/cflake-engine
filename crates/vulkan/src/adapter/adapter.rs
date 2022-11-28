use std::ffi::CStr;

use ash::vk::{
    self, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceMemoryProperties, PhysicalDeviceProperties,
    PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};

use crate::{Instance, Surface};

// Wrapper around a physical device
pub struct Adapter {
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) physical_device_memory_properties:
        PhysicalDeviceMemoryProperties,
    pub(crate) physical_device_features: PhysicalDeviceFeatures,
    pub(crate) physical_device_properties: PhysicalDeviceProperties,
    pub(crate) physical_device_surface_capabilities:
        SurfaceCapabilitiesKHR,
    pub(crate) physical_device_present_modes: Vec<PresentModeKHR>,
    pub(crate) physical_device_present_formats: Vec<SurfaceFormatKHR>,
    pub(crate) physical_device_queue_family_properties:
        Vec<vk::QueueFamilyProperties>,
    pub(crate) physical_device_queue_family_surface_supported:
        Vec<bool>,
}

impl Adapter {
    // Pick a physical device from the Vulkan instance
    pub unsafe fn pick(
        instance: &Instance,
        integrated: bool,
        surface: &Surface,
    ) -> Adapter {
        let devices =
            instance.instance.enumerate_physical_devices().unwrap();

        devices
            .iter()
            .map(|physical_device| {
                // We must first check if the device supports the surface
                let len = instance
                    .instance
                    .get_physical_device_queue_family_properties(*physical_device)
                    .len();
                let range = 0..len;

                let present_supported = range.into_iter().map(|i| {
                    surface
                        .surface_loader()
                        .get_physical_device_surface_support(
                            *physical_device, i as u32,
                            surface.surface()
                        ).unwrap_or_default()
                });

                (physical_device, present_supported)
            })
            .map(|(&physical_device, present_supported_per_family)| {
                // Get the features of the physical device
                let physical_device_features = instance
                    .instance
                    .get_physical_device_features(physical_device);

                // Get the properties of the physical device
                let physical_device_properties = instance
                    .instance
                    .get_physical_device_properties(physical_device);

                // Get the memory properties of the physical device
                let physical_device_memory_properties = instance
                    .instance
                    .get_physical_device_memory_properties(
                        physical_device,
                    );

                // Get the surface capabilities of the physical device
                let physical_device_surface_capabilities = surface.surface_loader()
                    .get_physical_device_surface_capabilities(physical_device, surface.surface()).unwrap();

                // Get the present modes of the physical device
                let physical_device_present_modes = surface.surface_loader()
                    .get_physical_device_surface_present_modes(physical_device, surface.surface()).unwrap();
                    
                // Get the supported formats of the physical device
                let physical_device_present_formats = surface.surface_loader()
                    .get_physical_device_surface_formats(physical_device, surface.surface()).unwrap();

                // Get the queue family properties of the physical device
                let physical_device_queue_family_properties = instance
                    .instance
                    .get_physical_device_queue_family_properties(physical_device);

                // Check each device family and see if we can present to it
                let physical_device_queue_family_surface_supported = 
                    present_supported_per_family.collect::<Vec<bool>>();

                // Convert the values to a simple adapter
                Adapter {
                    physical_device,
                    physical_device_memory_properties,
                    physical_device_features,
                    physical_device_properties,
                    physical_device_surface_capabilities,
                    physical_device_present_modes,
                    physical_device_present_formats,
                    physical_device_queue_family_properties,
                    physical_device_queue_family_surface_supported,
                }
            }).find(|adapter| adapter.is_physical_device_suitable(integrated))
            .expect("Could not find a suitable GPU to use!")
    }

    // Check wether or not a physical device is suitable for rendering
    // This checks the minimum requirements that we need to achieve to be able to render
    unsafe fn is_physical_device_suitable(
        &self,
        integrated: bool,
    ) -> bool {
        use vk::PhysicalDeviceType;
        let name = self.physical_device_properties.device_name;
        let _type = self.physical_device_properties.device_type;
        let surface = self.physical_device_surface_capabilities;
        let modes = self.physical_device_present_modes.as_slice();
        let formats = self.physical_device_present_formats.as_slice();
        let name = CStr::from_ptr(name.as_ptr()).to_str().unwrap();
        log::debug!("Checking if adapter {} is suitable..", name);

        // Check if double buffering is supported
        let double_buffering_supported = surface.min_image_count <= 2
            && surface.max_image_count >= 2;
        log::debug!(
            "Adapter Double Buffering: {}",
            double_buffering_supported
        );

        // Check if the present format is supported
        let format_supported = formats
            .iter()
            .find(|format| {
                let format_ =
                    format.format == vk::Format::B8G8R8A8_SRGB;
                let color_space_ = format.color_space
                    == vk::ColorSpaceKHR::SRGB_NONLINEAR;
                format_ && color_space_
            })
            .is_some();
        log::debug!(
            "Adapter Swapchain Format Supported: {}",
            format_supported
        );

        // Check if the minimum required present mode is supported
        let present_supported = modes
            .iter()
            .find(|&&present| {
                let relaxed =
                    present == vk::PresentModeKHR::FIFO_RELAXED;
                let immediate =
                    present == vk::PresentModeKHR::IMMEDIATE;
                relaxed || immediate
            })
            .is_some();

        // Check the device type
        let device_type_okay = if integrated {
            _type == PhysicalDeviceType::INTEGRATED_GPU
        } else {
            _type == PhysicalDeviceType::DISCRETE_GPU
        };
        log::debug!("Adapter Device Type: {:?}", _type);

        // All the checks must pass
        double_buffering_supported
            && format_supported
            && present_supported
            && device_type_okay
            && self.is_physical_device_suitable_additional()
    }

    // Additional requirements the adapter must meet to be able to use it properly
    unsafe fn is_physical_device_suitable_additional(&self) -> bool {
        true
    }
}
