use crate::{Adapter, Device, Instance, Queue, Surface};
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
    pub fn new(
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
        let present_mode = unsafe {
            Self::pick_presentation_mode(surface, adapter, vsync)
        };
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
            &instance.raw(),
            &device.raw(),
        );
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Could not create the swapchain")
        };

        // Create the image handles
        let swapchain_images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain).unwrap()
        };
        let min = adapter.surface.surface_capabilities.min_image_count
            as usize;
        log::debug!(
            "Swapchain contains {} images. {} more than the minimum",
            swapchain_images.len(),
            swapchain_images.len() - min
        );

        // Semaphore that is signaled whenever we have a new available image
        let image_available_semaphore =
            unsafe { device.create_semaphore() };

        // Semaphore that is signaled when we finished rendering
        let rendering_finished_semaphore =
            unsafe { device.create_semaphore() };

        // Fence that is signaled when we finished rendered
        let rendering_finished_fence =
            unsafe { device.create_fence() };

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
                adapter.surface.surface_capabilities.min_image_count,
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
    // Get the next free image that we can render to
    pub unsafe fn acquire_next_image(
        &self,
    ) -> Option<(u32, vk::Image)> {
        let err = self.loader.acquire_next_image(
            self.raw,
            u64::MAX,
            self.image_available_semaphore,
            vk::Fence::null(),
        );

        match err {
            Ok((index, _)) => {
                Some((index, self.images[index as usize]))
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(_) => None,
        }
    }

    // Present an image to the swapchain and make sure it will wait on the correspoding semaphore
    // This will return an Option telling us if we should recreate the swapchain or not
    pub unsafe fn present(
        &self,
        queue: &Queue,
        index: (u32, vk::Image),
    ) -> Option<()> {
        // Wait until the command buffers finished executing so we can present the image
        let present_info = *vk::PresentInfoKHR::builder()
            .swapchains(&[self.raw])
            .wait_semaphores(&[self.image_available_semaphore])
            .image_indices(&[index.0]);

        // Present the image to the screen
        let err =
            self.loader.queue_present(queue.queue, &present_info);

        match err {
            Ok(_) => Some(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(_) => Some(()),
        }
    }

    // Get the internal surface format of the swapchain
    pub fn format(&self) -> vk::Format {
        self.format.format
    }

    // Get the internal color space used by the swapchain
    pub fn color_space(&self) -> vk::ColorSpaceKHR {
        self.format.color_space
    }

    // Recreate the swapchain with some new dimensions
    pub unsafe fn recreate(
        &self,
        device: &Device,
        dimensions: vek::Extent2<u32>,
    ) {
    }
}
