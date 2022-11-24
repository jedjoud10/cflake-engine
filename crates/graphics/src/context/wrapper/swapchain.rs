use crate::{
    Adapter, Device, FrameRateLimit, Instance, Surface,
    WindowSettings,
};
use ash::vk::{self};

// Wrapper around the vulkan swapchain
pub(crate) struct Swapchain {
    pub(crate) loader: ash::extensions::khr::Swapchain,
    pub(crate) raw: vk::SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) extent: vk::Extent2D,
    pub(crate) rendering_finished_semaphore: vk::Semaphore,
    pub(crate) rendering_finished_fence: vk::Fence,
    pub(crate) image_available_semaphore: vk::Semaphore,
    pub(crate) format: vk::SurfaceFormatKHR,
    pub(crate) present_mode: vk::PresentModeKHR,
}

impl Swapchain {
    pub(crate) unsafe fn destroy(self, device: &Device) {
        device.device.device_wait_idle().unwrap();
        device
            .device
            .destroy_semaphore(self.image_available_semaphore, None);
        device.device.destroy_semaphore(
            self.rendering_finished_semaphore,
            None,
        );
        device
            .device
            .destroy_fence(self.rendering_finished_fence, None);
        self.loader.destroy_swapchain(self.raw, None);
    }
}

// Create the image swapchain that we will present to the screen
pub(crate) unsafe fn create_swapchain(
    adapter: &Adapter,
    surface: &Surface,
    device: &Device,
    instance: &Instance,
    window: &winit::window::Window,
    window_settings: &WindowSettings,
) -> Swapchain {
    // Get the supported surface formats khr
    let format = pick_surface_format(surface, adapter);

    // Create the swapchain image size
    let extent = *vk::Extent2D::builder()
        .height(window.inner_size().height)
        .width(window.inner_size().width);

    // Pick the most appropriate present mode
    let present_mode =
        pick_presentation_mode(surface, adapter, window_settings);

    // Create the swap chain create info
    let swapchain_create_info = create_swapchain_create_info(
        surface,
        format,
        extent,
        present_mode,
    );

    // Create the loader and the actual swapchain
    let swapchain_loader = ash::extensions::khr::Swapchain::new(
        &instance.instance,
        &device.device,
    );
    let swapchain = swapchain_loader
        .create_swapchain(&swapchain_create_info, None)
        .expect("Could not create the swapchain");

    // Create the image handles
    let swapchain_images =
        swapchain_loader.get_swapchain_images(swapchain).unwrap();

    // Semaphore that is signaled whenever we have a new available image
    let image_available_semaphore = device.create_semaphore();

    // Semaphore that is signaled when we finished rendering
    let rendering_finished_semaphore = device.create_semaphore();

    // Fence that is signaled when we finished rendering
    let rendering_finished_fence = device.create_fence();

    Swapchain {
        loader: swapchain_loader,
        raw: swapchain,
        images: swapchain_images,
        extent,
        rendering_finished_semaphore,
        rendering_finished_fence,
        image_available_semaphore,
        format,
        present_mode,
    }
}

// Create the swapchain create info
fn create_swapchain_create_info(
    surface: &Surface,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present: vk::PresentModeKHR,
) -> vk::SwapchainCreateInfoKHR {
    *vk::SwapchainCreateInfoKHR::builder()
        .surface(surface.surface)
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

// Pick the proper swapchain presentation mode
unsafe fn pick_presentation_mode(
    surface: &Surface,
    adapter: &Adapter,
    window_settings: &WindowSettings,
) -> vk::PresentModeKHR {
    // Fetch all the present modes
    let modes = surface
        .surface_loader
        .get_physical_device_surface_present_modes(
            adapter.physical_device,
            surface.surface,
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
        modes.into_iter().find(|m| *m == vk::PresentModeKHR::IMMEDIATE)
            .expect("Could not find an appropriate NON-VSYNC present mode")
    }
}

// Pick the proper surface format for presenting
unsafe fn pick_surface_format(
    surface: &Surface,
    adapter: &Adapter,
) -> vk::SurfaceFormatKHR {
    surface
        .surface_loader
        .get_physical_device_surface_formats(
            adapter.physical_device,
            surface.surface,
        )
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
