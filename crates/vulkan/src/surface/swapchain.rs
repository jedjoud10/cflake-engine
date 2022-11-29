use crate::{Adapter, Device, FamilyType, Instance, Queues, Surface};
use ash::vk::{self};
use parking_lot::Mutex;

// Wrapper around the vulkan swapchain
pub struct Swapchain {
    // Swapchain
    pub(super) loader: ash::extensions::khr::Swapchain,
    pub(super) raw: vk::SwapchainKHR,

    // Image data
    pub(super) images: Vec<vk::Image>,
    pub(super) extent: vk::Extent2D,

    // Synchronization
    pub(super) rendering_finished_semaphore: vk::Semaphore,
    pub(super) rendering_finished_fence: Mutex<vk::Fence>,
    pub(super) image_available_semaphore: vk::Semaphore,

    // Format and present mode
    pub(super) format: vk::SurfaceFormatKHR,
    pub(super) present_mode: vk::PresentModeKHR,
}

impl Swapchain {
    // Create the image swapchain that we will present to the screen
    pub unsafe fn new(
        adapter: &Adapter,
        surface: &Surface,
        device: &Device,
        instance: &Instance,
        window: &winit::window::Window,
        vsync: bool,
    ) -> Swapchain {
        // Get the supported surface formats khr
        let format = *vk::SurfaceFormatKHR::builder()
            .format(vk::Format::B8G8R8A8_SRGB)
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR);

        // Create the swapchain image size
        let extent = *vk::Extent2D::builder()
            .height(window.inner_size().height)
            .width(window.inner_size().width);

        // Pick the most appropriate present mode
        let present_mode =
            Self::pick_presentation_mode(surface, adapter, vsync);
        log::info!("Picked the presentation mode {:?}", present_mode);

        // Create the swap chain create info
        let swapchain_create_info =
            Self::create_swapchain_create_info(
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
        log::info!(
            "Swapchain contains {} images. {} more than the minimum",
            swapchain_images.len(),
            swapchain_images.len() - 2
        );

        // Semaphore that is signaled whenever we have a new available image
        let image_available_semaphore = device.create_semaphore();

        // Semaphore that is signaled when we finished rendering
        let rendering_finished_semaphore = device.create_semaphore();

        Swapchain {
            loader: swapchain_loader,
            raw: swapchain,
            images: swapchain_images,
            extent,
            rendering_finished_semaphore,
            rendering_finished_fence: Mutex::new(vk::Fence::null()),
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
        vsync: bool,
    ) -> vk::PresentModeKHR {
        // Fetch all the present modes
        let modes = surface
            .surface_loader
            .get_physical_device_surface_present_modes(
                adapter.physical_device,
                surface.surface,
            )
            .unwrap();

        if vsync {
            // VSYNC = Mailbox -> Fifo -> Fifo Relaxed
            modes
                .into_iter()
                .filter(|m| *m != vk::PresentModeKHR::IMMEDIATE)
                .min_by_key(|mode| *mode)
                .unwrap()
        } else {
            // No VSYNC = Immediate
            modes
                .into_iter()
                .find(|m| *m == vk::PresentModeKHR::IMMEDIATE)
                .unwrap()
        }
    }

    // Destroy the swapchain
    pub unsafe fn destroy(self, device: &Device) {
        device.device.device_wait_idle().unwrap();
        device
            .device
            .destroy_semaphore(self.image_available_semaphore, None);

        device.device.destroy_semaphore(
            self.rendering_finished_semaphore,
            None,
        );

        device.device.destroy_fence(
            *self.rendering_finished_fence.lock(),
            None,
        );
        self.loader.destroy_swapchain(self.raw, None);
    }
}

impl Swapchain {
    // Get the next free image that we can render to
    pub unsafe fn aquire(&self) -> (u32, vk::Image) {
        let (index, _) = self
            .loader
            .acquire_next_image(
                self.raw,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .unwrap();
        (index, self.images[index as usize])
    }

    // Execute some commands on the specific image
    pub unsafe fn render(
        &self,
        device: &Device,
        queues: &Queues,
        image: (u32, vk::Image),
    ) {
        let present = queues.family(FamilyType::Present);

        // Fetch the command pool of the current thread
        let pool = present.aquire_pool();

        // Create a new recorder (or fetches an current one)
        let recorder = pool.aquire_recorder(
            device,
            vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        );

        // Image subresource range
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);

        // Reset the presented image layout to be able to clear it
        let present_to_clear = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::MEMORY_READ)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(present.index())
            .dst_queue_family_index(present.index())
            .image(image.1)
            .subresource_range(*subresource_range);

        // Convert the clear image layout to be able to present it
        let clear_to_present = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_queue_family_index(present.index())
            .dst_queue_family_index(present.index())
            .image(image.1)
            .subresource_range(*subresource_range);

        // Set the clear color of the image view
        let mut clear_color_value = vk::ClearColorValue::default();
        clear_color_value.float32 = [0.1; 4];

        // Convert image layouts and wait
        device.device.cmd_pipeline_barrier(
            recorder.cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*present_to_clear],
        );

        // Clear the color of the image
        device.device.cmd_clear_color_image(
            recorder.cmd,
            image.1,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &clear_color_value,
            &[*subresource_range],
        );

        // Convert image layouts and wait
        device.device.cmd_pipeline_barrier(
            recorder.cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*clear_to_present],
        );

        *self.rendering_finished_fence.lock() = pool
            .submit_recorders_from_iter(
                device,
                &[recorder],
                &[self.rendering_finished_semaphore],
                &[self.image_available_semaphore],
                &[],
                //self.rendering_finished_fence
            );
    }

    /*
    // Wait until we have a presentable image we can write to
    let submit_info = *vk::SubmitInfo::builder()
        .wait_semaphores(&[
            self.swapchain.image_available_semaphore
        ])
        .signal_semaphores(&[
            self.swapchain.rendering_finished_semaphore
        ]);

    // Submit the command buffers
    let queue = self
        .device
        .device
        .get_device_queue(self.queues.graphics(), 0);
    self.device
        .queue_submit(
            queue,
            &[submit_info],
            swapchain.rendering_finished_fence,
        )
        .unwrap();

    // Wait until the command buffers finished executing so we can present the image
    let present_info = *vk::PresentInfoKHR::builder()
        .swapchains(&[self.swapchain.raw])
        .wait_semaphores(&[
            self.swapchain.rendering_finished_semaphore
        ])
        .image_indices(&[image_index]);

    // Present the image to the screen
    self.swapchain
    device
        .loader
        .queue_present(queue, &present_info)
        .unwrap();

    */

    // Present the given image (assuming it was already stored)
    // This will wait until rendering has completed
    pub unsafe fn present(
        &self,
        device: &Device,
        queues: &Queues,
        image: (u32, vk::Image),
    ) {
        let present_info = *vk::PresentInfoKHR::builder()
            .swapchains(&[self.raw])
            .wait_semaphores(&[self.rendering_finished_semaphore])
            .image_indices(&[image.0]);

        // Get the present queue
        let queue = queues.family(FamilyType::Present).queue();

        // Present the image to the screen
        let _suboptimal =
            self.loader.queue_present(queue, &present_info).unwrap();

        // Wait till the last frame finished rendering
        device
            .device
            .wait_for_fences(
                &[*self.rendering_finished_fence.lock()],
                true,
                u64::MAX,
            )
            .unwrap();

        /*
        let present = queues.family(FamilyType::Present);
        let pool = present.aquire_specific_pool(0).unwrap();
        pool.reset(device);
        */
        /*
        device.device
            .reset_fences(&[*self.rendering_finished_fence.lock()])
            .unwrap();
        */
    }
}
