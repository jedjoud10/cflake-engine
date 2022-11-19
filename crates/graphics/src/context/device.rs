use std::ffi::CStr;
use ash::vk::{self, PhysicalDeviceProperties, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, PhysicalDevice, DeviceQueueCreateInfo, DeviceCreateInfo};
use super::GraphicSettings;

// Wrapper around API and Driver version for physical devices and
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeviceVersion {
    pub api: u32,
    pub driver: u32,
}

// This is a wrapper resource around a Vulkan physical device and a logical device
// This will also contain all the queues that we must use when creating something 
pub struct Device {
    logical_device: ash::Device,
    physical_device: PhysicalDevice,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    physical_device_features: PhysicalDeviceFeatures,
    physical_device_properties: PhysicalDeviceProperties,
}

impl Device {
    // Function that will be called per physical device to check if we can use it
    unsafe fn find_physical_device(
        physical_device: &PhysicalDevice,
        feature: &PhysicalDeviceFeatures,
        properties: &PhysicalDeviceProperties,
        graphic_settings: &GraphicSettings
    ) -> bool {
        properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    }

    // Create a new logical device and pick a physical device
    pub(crate) unsafe fn new(
        graphic_settings: &GraphicSettings,
        instance: &ash::Instance,
        entry: &ash::Entry,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
    ) -> Self {        
        // Get a list of devices that we can use
        let devices = instance.enumerate_physical_devices().unwrap();
        let (physical_device,
            physical_device_features,
            physical_device_properties
        ) = devices
            .iter()
            .map(|device| {
                let features = instance.get_physical_device_features(*device);
                let properties = instance.get_physical_device_properties(*device);
                (*device, features, properties)
            })
            .find(|(a, b, c)| 
                Self::find_physical_device(a, b, c, graphic_settings))
            .expect("Could not find a suitable GPU to use!");

        // Get physical device memory properties
        let physical_device_memory_properties =
            instance.get_physical_device_memory_properties(physical_device);

        // Get the queue family from this physical device
        let queues = 
            instance.get_physical_device_queue_family_properties(physical_device);

        // Find the index for the graphics queue family
        let graphics_queue_index = queues
            .iter()
            .enumerate()
            .position(|(i, props)| {
                props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader
                        .get_physical_device_surface_support(physical_device, i as u32, *surface)
                        .unwrap()
            })
            .expect("Could not find the graphics queue") as u32;

        // Specify the logical device's queue info
        let queue_create_info = DeviceQueueCreateInfo::builder()
            .queue_priorities(&[1.0f32])
            .queue_family_index(graphics_queue_index);
        let infos = [queue_create_info.build()];

        // Create logical device create info
        let logical_device_extensions = graphic_settings.logical_device_extensions.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        let logical_device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&infos)
            .enabled_extension_names(&logical_device_extensions)
            .enabled_features(&physical_device_features);

        // Create the logical device
        let logical_device = instance
            .create_device(physical_device, &logical_device_create_info, None)
            .expect("Could not create the logical device");

        Self { 
            logical_device,
            physical_device,
            physical_device_memory_properties,
            physical_device_features,
            physical_device_properties,
        }
    }

    // Get the name of the chosen physical device
    pub fn name(&self) -> &str {
        unsafe { 
            CStr::from_ptr(self.physical_device_properties.device_name.as_ptr())
                .to_str()
                .unwrap()
        }
    }
    
    // Get the driver version of the physical device
    pub fn driver_version(&self) -> u32 {
        self.physical_device_properties.driver_version
    }

    // Get the API version of the physical device
    pub fn api_version(&self) -> u32 {
        self.physical_device_properties.api_version
    } 

    // Destroy the physical device and logical device and queues and memory AAA
    pub(crate) unsafe fn destroy(self) {
        self.logical_device.destroy_device(None);
    }
}