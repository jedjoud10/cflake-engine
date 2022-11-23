use super::{FrameRateLimit, GraphicSettings, WindowSettings};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk::{
        self, DeviceCreateInfo, DeviceQueueCreateInfo,
        PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceMemoryProperties, PhysicalDeviceProperties,
    },
    Entry, Instance,
};
use std::{
    borrow::Cow,
    ffi::{c_void, CStr},
};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Debug callback that is invoked from the debug messenger
#[cfg(debug_assertions)]
pub(super) unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    cvoid: *mut c_void,
) -> u32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 =
        callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null()
    {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name)
            .to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

// Create a single simple semaphore
pub(super) unsafe fn create_semaphore(
    device: &ash::Device,
) -> vk::Semaphore {
    device.create_semaphore(&Default::default(), None).unwrap()
}

// Create a single simple fence
pub(super) unsafe fn create_fence(device: &ash::Device) -> vk::Fence {
    device.create_fence(&Default::default(), None).unwrap()
}

// Create the swapchain create info
pub(super) fn create_swapchain_create_info(
    surface: vk::SurfaceKHR,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present: vk::PresentModeKHR,
) -> vk::SwapchainCreateInfoKHR {
    *vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(2)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .image_usage(
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
        )
        .clipped(true)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .old_swapchain(vk::SwapchainKHR::null())
        .present_mode(present)
}

// Create the debug utils create info
pub(super) unsafe fn create_debug_messenger_create_info(
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    *vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(debug_callback))
}

// Pick the proper swapchain presentation mode
pub(super) unsafe fn pick_presentation_mode(
    surface_loader: &Surface,
    physical_device: PhysicalDevice,
    surface: vk::SurfaceKHR,
    window_settings: &WindowSettings,
) -> vk::PresentModeKHR {
    // Fetch all the present modes
    let modes = surface_loader
        .get_physical_device_surface_present_modes(
            physical_device,
            surface,
        )
        .unwrap();

    if matches!(window_settings.limit, FrameRateLimit::VSync) {
        // VSYNC = Mailbox -> Fifo -> Fifo Relaxed
        modes
            .into_iter()
            .filter(|m| *m != vk::PresentModeKHR::IMMEDIATE)
            .min_by_key(|mode| *mode)
            .expect(
                "Could not find an appropriate VSYNC present mode",
            )
    } else {
        // No VSYNC = Immediate
        modes.into_iter()
            .filter(|m| *m == vk::PresentModeKHR::IMMEDIATE)
            .next()
            .expect("Could not find an appropriate NON-VSYNC present mode")
    }
}

// Pick the proper surface format for presenting
pub(super) unsafe fn pick_surface_format(
    surface_loader: &Surface,
    physical_device: PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> vk::SurfaceFormatKHR {
    surface_loader
        .get_physical_device_surface_formats(physical_device, surface)
        .unwrap()
        .into_iter()
        .find(|surface_format| {
            let fmt =
                surface_format.format == vk::Format::B8G8R8A8_SRGB;
            let cs = surface_format.color_space
                == vk::ColorSpaceKHR::SRGB_NONLINEAR;
            fmt && cs
        })
        .expect("Could not find an appropriate present format!")
}

// Find a queue that supports the specific flags
pub(super) unsafe fn pick_queue_family(
    queue_families: &[vk::QueueFamilyProperties],
    surface_loader: &Surface,
    physical_device: PhysicalDevice,
    surface: vk::SurfaceKHR,
    supports_presenting: bool,
    flags: vk::QueueFlags,
) -> u32 {
    queue_families
        .iter()
        .enumerate()
        .position(|(i, props)| {
            // Check if the queue family supporsts the flags
            let flags = props.queue_flags.contains(flags);

            // If the queue we must fetch must support presenting, fetch the physical device properties
            let presenting = !supports_presenting
                || surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                        surface,
                    )
                    .unwrap();

            flags && presenting
        })
        .expect("Could not find the graphics queue") as u32
}

// Check wether or not a physical device is suitable for rendering
pub(super) unsafe fn is_physical_device_suitable(
    surface_loader: &ash::extensions::khr::Surface,
    surface: &ash::vk::SurfaceKHR,
    instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    physical_device_features: &PhysicalDeviceFeatures,
    physical_device_properties: &PhysicalDeviceProperties,
    graphic_settings: &GraphicSettings,
) -> bool {
    physical_device_properties.device_type
        == vk::PhysicalDeviceType::DISCRETE_GPU
}

// Pick a physical device from the Vulkan instance
pub(super) unsafe fn pick_physical_device(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
    graphic_settings: &GraphicSettings,
) -> (
    PhysicalDevice,
    PhysicalDeviceFeatures,
    PhysicalDeviceProperties,
) {
    let devices = instance.enumerate_physical_devices().unwrap();
    let (
        physical_device,
        physical_device_features,
        physical_device_properties,
    ) = devices
        .iter()
        .map(|device| {
            // Get the features of the physical device
            let features =
                instance.get_physical_device_features(*device);

            // Get the properties of the physical device
            let properties =
                instance.get_physical_device_properties(*device);
            (*device, features, properties)
        })
        .find(
            |(
                physical_device,
                physical_device_features,
                physical_device_properties,
            )| {
                // Pick the first device that is suitable for rendering
                is_physical_device_suitable(
                    surface_loader,
                    &surface,
                    instance,
                    physical_device,
                    physical_device_features,
                    physical_device_properties,
                    graphic_settings,
                )
            },
        )
        .expect("Could not find a suitable GPU to use!");

    // Return the picked physical device
    (
        physical_device,
        physical_device_features,
        physical_device_properties,
    )
}
