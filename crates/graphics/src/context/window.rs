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
pub(crate) struct Swapchain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    image_index: u32,
    image_available_semaphore: vk::Semaphore,
    image_available_fence: vk::Fence,
}

// A window is what we will draw to at the end of each frame
pub struct Window {
    pub(crate) settings: WindowSettings,
    pub(crate) raw: winit::window::Window,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) surface_loader: ash::extensions::khr::Surface,

    // Swapchain for rendering
    pub(crate) swapchain: Option<Swapchain>,
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
        command_pool: &vk::CommandPool,
        command_buffers: &mut Vec<vk::CommandBuffer>,
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
            .clipped(true)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .old_swapchain(vk::SwapchainKHR::null())
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
        
        // Create a semaphore for swapchain presentation
        let image_available_semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let image_available_semaphore = logical_device
            .create_semaphore(&image_available_semaphore_create_info, None)
            .unwrap();

        // Create a fence for swapchain presentation
        let image_available_fence_create_info = vk::FenceCreateInfo::builder();
        let image_available_fence = logical_device
            .create_fence(&image_available_fence_create_info, None)
            .unwrap();

        // Create a multiple command buffer
        let command_buffer_allocation_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool)
            .command_buffer_count(swapchain_image_views.len() as u32)
            .level(vk::CommandBufferLevel::PRIMARY);
        let swapchain_command_buffers = logical_device.allocate_command_buffers(&command_buffer_allocation_info).unwrap();
        command_buffers.extend(swapchain_command_buffers.clone());

        // Record each command buffer separately
        for i in 0..swapchain_image_views.len() {
            // Get command buffer for the correspoding image view
            let command_buffer = swapchain_command_buffers[i];
            let image_view = swapchain_image_views[i];
            let image = swapchain_images[i];
            let image_layout = vk::ImageLayout::GENERAL;

            
            // Record the command buffer and set the clear frame
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
            
            // Start recording the command buffer
            logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();

            // Set the clear color of the image view
            let mut clear_color_value = vk::ClearColorValue::default();
            clear_color_value.float32 = [1.0, 1.0, 1.0, 1.0];

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);

            // Command to record
            logical_device.cmd_clear_color_image(command_buffer, image, image_layout, &clear_color_value, &[*subresource_range]);

            // Stop recording the command buffer
            logical_device.end_command_buffer(command_buffer).unwrap();
        }
        

        self.swapchain = Some(Swapchain {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            image_index: 0,
            image_available_semaphore,
            image_available_fence,
        })
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
