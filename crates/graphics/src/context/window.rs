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
    swapchain_command_buffers: Vec<vk::CommandBuffer>,
    image_index: u32,
    rendering_finished_semaphore: vk::Semaphore,
    image_available_semaphore: vk::Semaphore,
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
        graphics_family_index: u32,
        command_buffers: &mut Vec<vk::CommandBuffer>,
    ) {
        // Get the supported surface formats khr
        let format = self
            .surface_loader
            .get_physical_device_surface_formats(
                *physical_device,
                self.surface,
            )
            .unwrap()
            .into_iter()
            .find(|surface_format| {
                let fmt = surface_format.format
                    == vk::Format::B8G8R8A8_SRGB;
                let cs = surface_format.color_space
                    == vk::ColorSpaceKHR::SRGB_NONLINEAR;
                fmt && cs
            })
            .expect("Could not find an appropriate present format!");

        // Create the swapchain image size
        let extent = *vk::Extent2D::builder()
            .height(self.raw.inner_size().height)
            .width(self.raw.inner_size().width);

        // Create the swap chain create info
        let swapchain_create_info =
            vk::SwapchainCreateInfoKHR::builder()
                .surface(self.surface)
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
                .present_mode(vk::PresentModeKHR::IMMEDIATE);

        // Create the loader and the actual swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(
            instance,
            logical_device,
        );
        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Could not create the swapchain");

        // Create the image handles
        let swapchain_images =
            swapchain_loader.get_swapchain_images(swapchain).unwrap();

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
                let image_view_create_info =
                    vk::ImageViewCreateInfo::builder()
                        .image(*image)
                        .components(components)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format.format)
                        .subresource_range(subresource_range);

                logical_device
                    .create_image_view(&image_view_create_info, None)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let image_available_semaphore_create_info =
            vk::SemaphoreCreateInfo::builder();
        let image_available_semaphore = logical_device
            .create_semaphore(
                &image_available_semaphore_create_info,
                None,
            )
            .unwrap();

        let rendering_finished_semaphore_create_info =
            vk::SemaphoreCreateInfo::builder();
        let rendering_finished_semaphore = logical_device
            .create_semaphore(&rendering_finished_semaphore_create_info, None)
            .unwrap();

        // Create a multiple command buffer
        let command_buffer_allocation_info =
            vk::CommandBufferAllocateInfo::builder()
                .command_pool(*command_pool)
                .command_buffer_count(
                    swapchain_image_views.len() as u32
                )
                .level(vk::CommandBufferLevel::PRIMARY);
        let swapchain_command_buffers = logical_device
            .allocate_command_buffers(&command_buffer_allocation_info)
            .unwrap();
        command_buffers.extend(swapchain_command_buffers.clone());

        // Record each command buffer separately
        for i in 0..swapchain_image_views.len() {
            // Get command buffer for the correspoding image view
            let command_buffer = swapchain_command_buffers[i];
            let image = swapchain_images[i];

            // Record the command buffer and set the clear frame
            let command_buffer_begin_info =
                vk::CommandBufferBeginInfo::builder().flags(
                    vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                );

            // Start recording the command buffer
            logical_device
                .begin_command_buffer(
                    command_buffer,
                    &command_buffer_begin_info,
                )
                .unwrap();

            // Image subresource range
            let subresource_range =
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1);

            let present_to_clear = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(graphics_family_index)
                .dst_queue_family_index(graphics_family_index)
                .image(image)
                .subresource_range(*subresource_range);

            let clear_to_present = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(graphics_family_index)
                .dst_queue_family_index(graphics_family_index)
                .image(image)
                .subresource_range(*subresource_range);

            // Set the clear color of the image view
            let mut clear_color_value =
                vk::ClearColorValue::default();
            clear_color_value.float32 = [1.0, 1.0, 1.0, 1.0];

            logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*present_to_clear],
            );

            logical_device.cmd_clear_color_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color_value,
                &[*subresource_range],
            );

            logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*clear_to_present],
            );

            // Stop recording the command buffer
            logical_device
                .end_command_buffer(command_buffer)
                .unwrap();
        }

        self.swapchain = Some(Swapchain {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            image_index: 0,
            image_available_semaphore,
            rendering_finished_semaphore,
            swapchain_command_buffers,
        })
    }

    // Draw the main window swapchain sheize
    pub(crate) unsafe fn draw(&mut self, device: &super::Device) {
        if let Some(swapchain) = &mut self.swapchain {
            let (image_index, b) = swapchain
                .swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    u64::MAX,
                    swapchain.image_available_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();

            let command_buffer = swapchain.swapchain_command_buffers
                [image_index as usize];
            let submit_info = *vk::SubmitInfo::builder()
                .wait_semaphores(&[swapchain.image_available_semaphore])
                .signal_semaphores(&[swapchain.rendering_finished_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::TRANSFER])
                .command_buffers(&[command_buffer]);

            let queue = device
                .logical_device
                .get_device_queue(device.graphics_queue_index, 0);
            device
                .logical_device
                .queue_submit(
                    queue,
                    &[submit_info],
                    vk::Fence::null(),
                )
                .unwrap();

            let present_info = *vk::PresentInfoKHR::builder()
                .swapchains(&[swapchain.swapchain])
                .wait_semaphores(&[swapchain.rendering_finished_semaphore])
                .image_indices(&[image_index]);

            swapchain
                .swapchain_loader
                .queue_present(queue, &present_info)
                .unwrap();
        }
    }

    // Destroy the window after we've done using it
    pub(crate) unsafe fn destroy(
        mut self,
        device: &super::Device,
    ) {
        device.logical_device.device_wait_idle().unwrap();

        let swapchain = self.swapchain.take().unwrap();
        device.logical_device.destroy_semaphore(
            swapchain.image_available_semaphore,
            None,
        );
        device.logical_device.destroy_semaphore(
            swapchain.rendering_finished_semaphore,
            None
        );
        swapchain
            .swapchain_loader
            .destroy_swapchain(swapchain.swapchain, None);
        for image_view in swapchain.swapchain_image_views {
            device.logical_device.destroy_image_view(image_view, None);
        }
        self.surface_loader.destroy_surface(self.surface, None);
    }
}
