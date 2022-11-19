use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    VSync,
    Limited(u32),
    Umlimited,
}

// Window setting that will tell Winit how to create the window
#[derive(Clone)]
pub struct WindowSettings {
    pub title: String,
    pub fullscreen: bool,
    pub limit: FrameRateLimit,
}

// Internal swapchain data wrapper
struct Swapchain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    /*
    image_index: u32,
    image_available_semaphore: vk::Semaphore,
    image_available_fence: vk::Fence,
    */
}

// A window is what we will draw to at the end of each frame
pub struct Window {
    settings: WindowSettings,
    raw: winit::window::Window,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,

    // Swapchain for rendering
    swapchain: Option<Swapchain>,
}

impl Window {
    // Create a new window using an event loop and it's settings
    pub(crate) unsafe fn new(
        window_settings: WindowSettings,
        raw: winit::window::Window,
        instance: &ash::Instance,
        entry: &ash::Entry,
    ) -> Self {
        // Get a window and display handle to the winit window
        let display_handle = raw.raw_display_handle();
        let window_handle = raw.raw_window_handle();

        // Create a surface loader and the surface itself
        let surface = ash_window::create_surface(
            &entry,
            &instance,
            display_handle,
            window_handle,
            None,
        )
        .unwrap();
        let surface_loader =
            ash::extensions::khr::Surface::new(&entry, &instance);

        Self {
            settings: window_settings,
            raw,
            surface,
            surface_loader,
            swapchain: None,
        }
    }

    // Only initialize the swapchain after we've created the main device
    // This will also initialize the swapchain images
    pub(crate) unsafe fn create_swapchain(
        &mut self,
        instance: &ash::Instance,
        entry: &ash::Entry,
        physical_device: &vk::PhysicalDevice,
        logical_device: &ash::Device,
    ) {
        // Get the supported surface formats khr
        let format = self
            .surface_loader
            .get_physical_device_surface_formats(*physical_device, self.surface)
            .unwrap()
            .into_iter()
            .find(|surface_format| {
                let fmt = surface_format.format == vk::Format::B8G8R8A8_SRGB;
                let cs = surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR;
                fmt && cs
            }).expect("Could not find an appropriate present format!");

        // Create the swapchain image size
        let extent = *vk::Extent2D::builder()
            .height(self.raw.inner_size().height)
            .width(self.raw.inner_size().width);

        // Create the swap chain create info
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(2)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .present_mode(vk::PresentModeKHR::IMMEDIATE);

        // Create the loader and the actual swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Could not create the swapchain");

        // Create the image handles
        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
        
        // Component mapping
        let components = *vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY);
        
        // Subresource range
        let subresource_range = *vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);
        
        // Create the image views
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| {
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .components(components)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .subresource_range(subresource_range);

                logical_device.create_image_view(&image_view_create_info, None).unwrap()
            })
            .collect::<Vec<_>>();

        self.swapchain = Some(Swapchain {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_views
        })
    }

    // Get access to the internal settings this window used during initialization
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get access to the internal raw winit window
    pub fn raw(&self) -> &winit::window::Window {
        &self.raw
    }

    // Get access to the raw KHR surface
    pub fn surface(&self) -> &ash::vk::SurfaceKHR {
        &self.surface
    } 

    // Get access to the raw KHR surface loader
    pub fn surface_loader(&self) -> &ash::extensions::khr::Surface {
        &self.surface_loader
    }

    // Destroy the window after we've done using it
    pub(crate) unsafe fn destroy(
        mut self,
        physical_device: &vk::PhysicalDevice,
        logical_device: &ash::Device
    ) {
        let swapchain = self.swapchain.take().unwrap();
        swapchain.swapchain_loader.destroy_swapchain(swapchain.swapchain, None);
        for image_view in swapchain.swapchain_image_views {
            logical_device.destroy_image_view(image_view, None);
        }
        self.surface_loader.destroy_surface(self.surface, None);
    }
}
