use std::ffi::CStr;

use ash::vk::{
    self, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceLimits, PhysicalDeviceProperties, PresentModeKHR,
    SurfaceCapabilitiesKHR, SurfaceFormatKHR, PhysicalDeviceType, PhysicalDeviceFeatures2, PhysicalDeviceVulkan13Features, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan11Properties, PhysicalDeviceVulkan12Properties, PhysicalDeviceVulkan13Properties, PhysicalDeviceProperties2,
};

use super::{Instance, Surface};

// Features supported by an adapter
pub struct AdapterFeatures {
    pub features: PhysicalDeviceFeatures,
    pub features11: PhysicalDeviceVulkan11Features,
    pub features12: PhysicalDeviceVulkan12Features,
    pub features13: PhysicalDeviceVulkan13Features,
}

// Properties of an adapter
pub struct AdapterProperties {
    pub name: String,
    pub api_version: String,
    pub device_type: PhysicalDeviceType,
    pub device_id: u32,
    pub vendor_id: u32,

    pub limits: PhysicalDeviceLimits,
    pub properties: PhysicalDeviceProperties,
    pub properties11: PhysicalDeviceVulkan11Properties,
    pub properties12: PhysicalDeviceVulkan12Properties,
    pub properties13: PhysicalDeviceVulkan13Properties,
}

// Swapchain data supported by the adapter
pub struct AdapterSurfaceProperties {
    pub present_modes: Vec<PresentModeKHR>,
    pub present_formats: Vec<SurfaceFormatKHR>,
    pub surface_capabilities: SurfaceCapabilitiesKHR,
}

// Queue family properties
pub struct AdapterQueueFamiliesProperties {
    pub queue_family_properties:
        Vec<vk::QueueFamilyProperties>,
    pub queue_family_nums: usize,
    pub queue_family_surface_supported: Vec<bool>,
}

// An adapter is a physical device that was chosen manually by the user
// For now, this Vulkan abstraction library can only handle one adapter per instance
pub struct Adapter {
    // Raw physical device
    raw: PhysicalDevice,
    
    // Properties and features
    pub(crate) features: AdapterFeatures,
    pub(crate) properties: AdapterProperties,
    pub(crate) surface: AdapterSurfaceProperties,
    pub(crate) families: AdapterQueueFamiliesProperties,
}

impl Adapter {
    // Create an adapter from it's raw physical device
    unsafe fn from_raw_parts(
        instance: &Instance,
        physical: PhysicalDevice,
        surface: &Surface,
    ) -> Adapter {
        // Get the features and capabilities
        let features = get_adapter_features(instance, &physical);
        let properties = get_adapter_properties(instance, &physical, surface);
        let families = get_adapter_queue_family_properties(instance, &physical, surface);
        let surface = get_adapter_surface_properties(instance, &physical, surface);

        Adapter {
            raw: physical,
            features,
            properties,
            surface,
            families,
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

                super::is_physical_device_suitable(
                    &adapter.features,
                    &adapter.properties,
                    &adapter.surface,
                    &adapter.families
                ).then_some(adapter)
            })
            .expect("Could not find a suitable GPU to use!");

        log::debug!("Using the adpater {:?} with API version {:?}", adapter.properties.name,  adapter.properties.api_version);
        adapter
    }

    // Get the underlying physical device
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.raw
    }

    // Get the name of the adapter
    pub fn name(&self) -> &str {
        &self.properties.name
    }

    // Get the device type of the adapter
    pub fn device_type(&self) -> vk::PhysicalDeviceType {
        self.properties.device_type
    }
    
    // Get the device ID as a u32
    pub fn device_id(&self) -> u32 {
        self.properties.device_id
    }

    // Get the vendor ID as a u32
    pub fn vendor_id(&self) -> u32 {
        self.properties.vendor_id
    }
}


// Get the adapter features of a physical device
unsafe fn get_adapter_features(instance: &Instance, physical: &PhysicalDevice) -> AdapterFeatures {
    let mut features11 = PhysicalDeviceVulkan11Features::default();
    let mut features12 = PhysicalDeviceVulkan12Features::default();
    let mut features13 = PhysicalDeviceVulkan13Features::default();
    let mut features = PhysicalDeviceFeatures2::builder()
        .features(PhysicalDeviceFeatures::default())
        .push_next(&mut features11)
        .push_next(&mut features12)
        .push_next(&mut features13);
    instance
        .instance
        .get_physical_device_features2(*physical, &mut features);

    AdapterFeatures {
        features: features.features,
        features11,
        features12,
        features13,
    }
}

// Get the adapter properties of a physical device
unsafe fn get_adapter_properties(instance: &Instance, physical: &PhysicalDevice, surface: &Surface) -> AdapterProperties {
    let mut properties11 = PhysicalDeviceVulkan11Properties::default();
    let mut properties12 = PhysicalDeviceVulkan12Properties::default();
    let mut properties13 = PhysicalDeviceVulkan13Properties::default();    
    let mut properties = *PhysicalDeviceProperties2::builder()
        .properties(PhysicalDeviceProperties::default())
        .push_next(&mut properties11)
        .push_next(&mut properties12)
        .push_next(&mut properties13);
    instance
        .instance
        .get_physical_device_properties2(*physical, &mut properties);

    // Get the name of the physical device
    let name = CStr::from_ptr(properties.properties.device_name.as_ptr())
        .to_str()
        .unwrap()
        .to_owned();

    // Get the API version and create a string representing it
    let version = properties.properties.api_version;
    let api_version = format!("{}.{}.{}",
        vk::api_version_major(version),
        vk::api_version_minor(version),
        vk::api_version_patch(version)
    );

    AdapterProperties {
        name,
        api_version,
        device_type: properties.properties.device_type,
        device_id: properties.properties.device_id,
        vendor_id: properties.properties.vendor_id,
        properties: properties.properties,
        properties11,
        properties12,
        properties13,
        limits: properties.properties.limits,
    }
}

// Get the adapter surface properties
unsafe fn get_adapter_surface_properties(instance: &Instance, physical: &PhysicalDevice, surface: &Surface) -> AdapterSurfaceProperties {
    let present_modes = surface
        .surface_loader()
        .get_physical_device_surface_present_modes(
*            physical,
            surface.surface(),
        )
        .unwrap();
    let present_formats = surface
        .surface_loader()
        .get_physical_device_surface_formats(
            *physical,
            surface.surface(),
        )
        .unwrap();


    let surface_capabilities = surface
        .surface_loader()
        .get_physical_device_surface_capabilities(
            *physical,
            surface.surface(),
        )
        .unwrap();

    AdapterSurfaceProperties {
        present_modes,
        present_formats,
        surface_capabilities
    }
}

// Get the adapter queue family properties
unsafe fn get_adapter_queue_family_properties(instance: &Instance, physical: &PhysicalDevice, surface: &Surface) -> AdapterQueueFamiliesProperties {
    let queue_family_properties = instance
        .instance
        .get_physical_device_queue_family_properties(*physical);
    let queue_family_surface_supported = (0..queue_family_properties
        .len())
        .map(|i| {
            surface
                .surface_loader()
                .get_physical_device_surface_support(
                    *physical,
                    i as u32,
                    surface.surface(),
                )
                .unwrap()
        })
        .collect::<Vec<bool>>();

    AdapterQueueFamiliesProperties {
        queue_family_nums: queue_family_properties.len(),
        queue_family_properties,
        queue_family_surface_supported
    }
}