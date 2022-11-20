use std::ffi::CStr;
use ash::vk::{self, PhysicalDeviceProperties, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, PhysicalDevice, DeviceQueueCreateInfo, DeviceCreateInfo, PhysicalDeviceType};
use super::GraphicSettings;

// This is a wrapper resource around a Vulkan physical device and a logical device
// This will also contain all the queues that we must use when creating something 
pub struct Device {
    pub(crate) logical_device: ash::Device,
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    pub(crate) physical_device_features: PhysicalDeviceFeatures,
    pub(crate) physical_device_properties: PhysicalDeviceProperties,
    pub(crate) graphics_queue_index: u32,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    pub(crate) command_pool: vk::CommandPool,
}

impl Device {
    // Function that will be called per physical device to check if we can use it
    unsafe fn find_physical_device(
        surface_loader: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        instance: &ash::Instance,
        physical_device: &PhysicalDevice,
        physical_device_features: &PhysicalDeviceFeatures,
        physical_device_properties: &PhysicalDeviceProperties,
        graphic_settings: &GraphicSettings
    ) -> bool {
        let caps = surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface).unwrap();
        let min = graphic_settings.frames_in_swapchain < caps.max_image_count;
        let max = graphic_settings.frames_in_swapchain > caps.min_image_count;
        min && max
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
            .find(|(physical_device, physical_device_features, physical_device_properties)| 
                Self::find_physical_device(
                    surface_loader,
                    surface, 
                    instance,
                    physical_device,
                    physical_device_features,
                    physical_device_properties,
                    graphic_settings
            ))
            .expect("Could not find a suitable GPU to use!");

        // Get physical device memory properties
        let physical_device_memory_properties =
            instance.get_physical_device_memory_properties(physical_device);

        // Get the queue family from this physical device
        let queues = 
            instance.get_physical_device_queue_family_properties(physical_device);

        // Find the index for the graphics queue family
        let graphics_family_index = queues
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
            .queue_family_index(graphics_family_index);
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

        // Create a command pool for rendering
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_family_index);
        let command_pool = logical_device.create_command_pool(&command_pool_create_info, None).unwrap();
        Self { 
            logical_device,
            physical_device,
            physical_device_memory_properties,
            physical_device_features,
            physical_device_properties,
            graphics_queue_index: graphics_family_index,
            command_pool,
            command_buffers: Vec::default(),
        }
    }

    // Destroy the physical device and logical device and queues and memory AAA
    pub(crate) unsafe fn destroy(self) {
        self.logical_device.destroy_command_pool(self.command_pool, None);
        self.logical_device.destroy_device(None);
    }
}