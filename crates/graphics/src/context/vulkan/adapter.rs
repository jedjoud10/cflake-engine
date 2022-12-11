use std::ffi::CStr;

use ash::vk::{
    self, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceLimits, PhysicalDeviceProperties, PresentModeKHR,
    SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};

use crate::{Instance, Surface};

// An adapter is a physical device that was chosen manually by the user
// For now, this Vulkan abstraction library can only handle one adapter per instance
pub struct Adapter {
    // Raw physical device
    pub(crate) raw: PhysicalDevice,
    pub(crate) name: String,
    pub(crate) device_id: u32,
    pub(crate) vendor_id: u32,

    // Properties
    pub(crate) limits: PhysicalDeviceLimits,
    pub(crate) features: PhysicalDeviceFeatures,
    pub(crate) properties: PhysicalDeviceProperties,
    pub(crate) surface_capabilities: SurfaceCapabilitiesKHR,

    // Swapchain related
    pub(crate) present_modes: Vec<PresentModeKHR>,
    pub(crate) present_formats: Vec<SurfaceFormatKHR>,

    // Related to queue families
    pub(crate) queue_family_properties:
        Vec<vk::QueueFamilyProperties>,
    pub(crate) queue_family_nums: usize,
    pub(crate) queue_family_surface_supported: Vec<bool>,
}

impl Adapter {
    // Create an adapter from it's raw physical device
    unsafe fn from_raw_parts(
        instance: &Instance,
        physical_device: PhysicalDevice,
        surface: &Surface,
    ) -> Adapter {
        // Main features and capabilities
        let (
            features,
            properties,
            limits,
            surface_capabilities,
            name,
        ) = get_capabilities(instance, physical_device, surface);

        // Surface and swapchain related
        let (present_modes, present_formats) =
            get_swapchain_modes(surface, physical_device);

        // Queue family related
        let (queue_family_properties, queue_family_surface_supported) =
            get_queue_family_properties(
                instance,
                physical_device,
                surface,
            );

        Adapter {
            raw: physical_device,
            name,
            device_id: properties.device_id,
            vendor_id: properties.vendor_id,
            limits,
            features,
            properties,
            surface_capabilities,
            present_modes,
            present_formats,
            queue_family_nums: queue_family_properties.len(),
            queue_family_properties,
            queue_family_surface_supported,
        }
    }

    // Pick out a physical adapter automatically for the user
    // Pick a physical device from the Vulkan instance
    pub unsafe fn pick(
        instance: &Instance,
        surface: &Surface,
    ) -> Adapter {
        let devices =
            instance.instance.enumerate_physical_devices().unwrap();

        let adapter = devices
            .iter()
            .cloned()
            .find_map(|physical_device| {
                let adapter = Self::from_raw_parts(
                    instance,
                    physical_device,
                    surface,
                );
                adapter
                    .is_physical_device_suitable()
                    .then_some(adapter)
            })
            .expect("Could not find a suitable GPU to use!");

        log::debug!("Using the adpater {:?}", adapter.name);
        adapter
    }
}

// Get the queue family properties that are supported by this physical device
unsafe fn get_queue_family_properties(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface: &Surface,
) -> (Vec<vk::QueueFamilyProperties>, Vec<bool>) {
    let queue_family_properties = instance
        .instance
        .get_physical_device_queue_family_properties(physical_device);
    let queue_family_surface_supported = (0..queue_family_properties
        .len())
        .map(|i| {
            surface
                .surface_loader
                .get_physical_device_surface_support(
                    physical_device,
                    i as u32,
                    surface.surface,
                )
                .unwrap()
        })
        .collect::<Vec<bool>>();
    (queue_family_properties, queue_family_surface_supported)
}

// Get the swapchain modes and formats that are supported by this Adapter
unsafe fn get_swapchain_modes(
    surface: &Surface,
    physical_device: PhysicalDevice,
) -> (Vec<PresentModeKHR>, Vec<SurfaceFormatKHR>) {
    let present_modes = surface
        .surface_loader
        .get_physical_device_surface_present_modes(
            physical_device,
            surface.surface,
        )
        .unwrap();
    let present_formats = surface
        .surface_loader
        .get_physical_device_surface_formats(
            physical_device,
            surface.surface,
        )
        .unwrap();
    (present_modes, present_formats)
}

// Get the properties, features, limits, and the name of the physical device
unsafe fn get_capabilities(
    instance: &Instance,
    physical_device: PhysicalDevice,
    surface: &Surface,
) -> (
    PhysicalDeviceFeatures,
    PhysicalDeviceProperties,
    PhysicalDeviceLimits,
    SurfaceCapabilitiesKHR,
    String,
) {
    let features = instance
        .instance
        .get_physical_device_features(physical_device);
    let properties = instance
        .instance
        .get_physical_device_properties(physical_device);
    let limits = properties.limits;
    let surface_capabilities = surface
        .surface_loader
        .get_physical_device_surface_capabilities(
            physical_device,
            surface.surface,
        )
        .unwrap();
    let name = CStr::from_ptr(properties.device_name.as_ptr())
        .to_str()
        .unwrap()
        .to_owned();
    (features, properties, limits, surface_capabilities, name)
}
