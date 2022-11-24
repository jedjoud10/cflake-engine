use ash::vk::{
    self, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceMemoryProperties, PhysicalDeviceProperties,
};

use crate::{GraphicSettings, Instance, Surface};

// Wrapper around a physical device
pub(crate) struct Adapter {
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) physical_device_memory_properties:
        PhysicalDeviceMemoryProperties,
    pub(crate) physical_device_features: PhysicalDeviceFeatures,
    pub(crate) physical_device_properties: PhysicalDeviceProperties,
}

impl Adapter {
    // Pick a physical device from the Vulkan instance
    pub(crate) unsafe fn pick(
        instance: &Instance,
        _surface: &Surface,
        _graphic_settings: &GraphicSettings,
    ) -> Adapter {
        let devices =
            instance.instance.enumerate_physical_devices().unwrap();
        devices
            .iter()
            .map(|&physical_device| {
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

                // Convert the values to a simple adapter
                Adapter {
                    physical_device,
                    physical_device_memory_properties,
                    physical_device_features,
                    physical_device_properties,
                }
            })
            .find(|adapter| adapter.is_physical_device_suitable())
            .expect("Could not find a suitable GPU to use!")
    }

    // Check wether or not a physical device is suitable for rendering
    pub(crate) unsafe fn is_physical_device_suitable(&self) -> bool {
        self.physical_device_properties.device_type
            == vk::PhysicalDeviceType::DISCRETE_GPU
    }
}

