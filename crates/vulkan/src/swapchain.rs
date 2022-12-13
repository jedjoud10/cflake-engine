use crate::{Adapter, Device, Instance, Surface};
use ash::vk::{self};

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
    pub(super) rendering_finished_fence: vk::Fence,
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
        log::debug!(
            "Picked the presentation mode {:?}",
            present_mode
        );

        // Create the swap chain create info
        let swapchain_create_info =
            Self::create_swapchain_create_info(
                surface,
                adapter,
                format,
                extent,
                present_mode,
            );

        // Create the loader and the actual swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(
            &instance.instance,
            &device.raw(),
        );
        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Could not create the swapchain");

        // Create the image handles
        let swapchain_images =
            swapchain_loader.get_swapchain_images(swapchain).unwrap();
        let min =
            adapter.surface_capabilities.min_image_count as usize;
        log::debug!(
            "Swapchain contains {} images. {} more than the minimum",
            swapchain_images.len(),
            swapchain_images.len() - min
        );

        // Semaphore that is signaled whenever we have a new available image
        let image_available_semaphore = device.create_semaphore();

        // Semaphore that is signaled when we finished rendering
        let rendering_finished_semaphore = device.create_semaphore();

        // Fence that is signaled when we finished rendered
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
        adapter: &Adapter,
        format: vk::SurfaceFormatKHR,
        extent: vk::Extent2D,
        present: vk::PresentModeKHR,
    ) -> vk::SwapchainCreateInfoKHR {
        *vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface())
            .min_image_count(
                adapter.surface_capabilities.min_image_count,
            )
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
            .surface_loader()
            .get_physical_device_surface_present_modes(
                adapter.physical_device(),
                surface.surface(),
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
    pub unsafe fn destroy(&self, device: &Device) {
        device.raw().device_wait_idle().unwrap();

        device
            .raw()
            .destroy_semaphore(self.image_available_semaphore, None);
        device.raw().destroy_semaphore(
            self.rendering_finished_semaphore,
            None,
        );
        device
            .raw()
            .destroy_fence(self.rendering_finished_fence, None);
        self.loader.destroy_swapchain(self.raw, None);
    }
}

impl Swapchain {
    /*
    // Get the next free image that we can render to
    pub unsafe fn acquire_next_image(&self) -> u32 {
        let (index, _) = self
            .loader
            .acquire_next_image(
                self.raw,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            ).unwrap();
        index
    }

    // Execute some commands on the specific image
    // Present the given image (assuming it was already stored)
    pub unsafe fn render(&self, queue: &Queue, device: &Device) {
        let (index, _) = self
            .loader
            .acquire_next_image(
                self.raw,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )
            .unwrap();

        // Get a recorder for the present family
        let cmd = queue.aquire(device);

        /*


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
        device.raw().cmd_pipeline_barrier(
            recorder.cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*present_to_clear],
        );

        // Clear the color of the image
        device.raw().cmd_clear_color_image(
            recorder.cmd,
            image.1,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &clear_color_value,
            &[*subresource_range],
        );

        // Convert image layouts and wait
        device.raw().cmd_pipeline_barrier(
            recorder.cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*clear_to_present],
        );

        /*
        *self.rendering_finished_fence.lock() = pool
            .submit_recorders_from_iter(
                device,
                &[recorder],
                &[self.rendering_finished_semaphore],
                &[self.image_available_semaphore],
                &[],
                //self.rendering_finished_fence
            );
        */

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
            .raw()
            .raw()
            .get_device_queue(self.queues.graphics(), 0);
        self.raw()
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
        */

        /*
        let present_info = *vk::PresentInfoKHR::builder()
            .swapchains(&[self.raw])
            .wait_semaphores(&[self.rendering_finished_semaphores])
            .image_indices(&[image.0]);

        // Get the present queue
        let family = queues.family(vk::QueueFlags::empty(), true);
        let queue = family.queue();

        // Present the image to the screen
        let _suboptimal =
            self.loader.queue_present(queue, &present_info).unwrap();

        // Wait till the last frame finished rendering
        device
            .raw()
            .wait_for_fences(
                &[*self.rendering_finished_fences.lock()],
                true,
                u64::MAX,
            )
            .unwrap();

        let present = queues.family(FamilyType::Present);
        let pool = present.aquire_specific_pool(0).unwrap();
        pool.reset(device);
        device
            .raw()
            .reset_fences(&[*self.rendering_finished_fences.lock()])
            .unwrap();

            */
    }
    */
}
