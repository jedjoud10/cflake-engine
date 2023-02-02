use super::{Adapter, Device, Instance, Queue, Surface};
use ash::vk::{self};
use parking_lot::Mutex;

// Wrapper around the vulkan swapchain
pub struct Swapchain {
    // Swapchain
    pub(super) loader: ash::extensions::khr::Swapchain,
    pub(super) raw: Mutex<vk::SwapchainKHR>,

    // Image data
    pub(super) images: Mutex<Vec<(vk::Image, vk::ImageView)>>,
    pub(super) extent: Mutex<vek::Extent2<u32>>,

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
            .format(vk::Format::B8G8R8A8_UNORM)
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR);

        // Convert the window inner size to vek extent2d
        let extent = window.inner_size();
        let extent = vek::Extent2::new(extent.width, extent.height);

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
                vk::SwapchainKHR::null(),
            );

        // Create the loader and the actual swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(
            instance.raw(),
            device.raw(),
        );
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Could not create the swapchain")
        };

        // Create the image handles and image views
        let images = unsafe {
            Self::get_images_and_views(
                swapchain,
                &swapchain_loader,
                format.format,
                adapter,
                device,
            )
        };

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
            raw: Mutex::new(swapchain),
            images: Mutex::new(images),
            extent: Mutex::new(extent),
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
        extent: vek::Extent2<u32>,
        present: vk::PresentModeKHR,
        old_swapchain: vk::SwapchainKHR,
    ) -> vk::SwapchainCreateInfoKHR {
        // Create the swapchain image size
        let extent =
            *vk::Extent2D::builder().height(extent.h).width(extent.w);

        *vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface())
            .min_image_count(
                adapter
                    .surface_properties()
                    .surface_capabilities
                    .min_image_count,
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
            .old_swapchain(old_swapchain)
            .present_mode(present)
    }

    // Get the images and image views for the swapchain
    unsafe fn get_images_and_views(
        swapchain: vk::SwapchainKHR,
        loader: &ash::extensions::khr::Swapchain,
        format: vk::Format,
        adapter: &Adapter,
        device: &Device,
    ) -> Vec<(vk::Image, vk::ImageView)> {
        let swapchain_images = unsafe {
            loader.get_swapchain_images(swapchain).unwrap()
        };
        let min = adapter
            .surface_properties()
            .surface_capabilities
            .min_image_count as usize;
        log::debug!(
            "Swapchain contains {} images. {} more than the minimum",
            swapchain_images.len(),
            swapchain_images.len() - min
        );

        // Create the image views
        let image_views =
            swapchain_images.iter().map(|image| unsafe {
                device.create_image_view(
                    vk::ImageViewCreateFlags::empty(),
                    *image,
                    vk::ImageViewType::TYPE_2D,
                    format,
                    vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    },
                    vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                )
            });

        swapchain_images
            .iter()
            .cloned()
            .zip(image_views)
            .collect::<Vec<_>>()
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

        for (_, view) in &*self.images.lock() {
            device.destroy_image_view(*view);
        }

        device
            .raw()
            .destroy_fence(self.rendering_finished_fence, None);
        self.loader.destroy_swapchain(*self.raw.lock(), None);
    }
}

impl Swapchain {
    // Get the next free image that we can render to
    pub unsafe fn acquire_next_image(&self) -> Option<u32> {
        let err = self.loader.acquire_next_image(
            *self.raw.lock(),
            u64::MAX,
            self.image_available_semaphore,
            vk::Fence::null(),
        );

        match err {
            Ok((index, _)) => Some(index),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(_) => None,
        }
    }

    // Present an image to the swapchain and make sure it will wait on the correspoding semaphore
    // This will return an Option telling us if we should recreate the swapchain or not
    pub unsafe fn present(
        &self,
        queue: &Queue,
        device: &Device,
        index: u32,
    ) -> Option<()> {
        // Wait until the command buffers finished executing so we can present the image
        let present_info = *vk::PresentInfoKHR::builder()
            .swapchains(&[*self.raw.lock()])
            .wait_semaphores(&[self.image_available_semaphore])
            .image_indices(&[index]);

        // Present the image to the screen
        let err =
            self.loader.queue_present(queue.queue, &present_info);
        device.wait();

        match err {
            Ok(_) => Some(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(_) => None,
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

    // Get the current extent of the swapchain
    pub fn extent(&self) -> vek::Extent2<u32> {
        *self.extent.lock()
    }

    // Get the swapchain images and views
    pub fn images(&self) -> Vec<(vk::Image, vk::ImageView)> {
        self.images.lock().clone()
    }

    // Recreate the swapchain with some new dimensions
    pub unsafe fn resize(
        &self,
        adapter: &Adapter,
        device: &Device,
        surface: &Surface,
        dimensions: vek::Extent2<u32>,
    ) {
        log::warn!(
            "Recreating swapchain with new dimensions {dimensions}"
        );
        device.wait();

        let create_info = Self::create_swapchain_create_info(
            surface,
            adapter,
            self.format,
            dimensions,
            self.present_mode,
            *self.raw.lock(),
        );

        // Create a new swapchain
        let swapchain = unsafe {
            self.loader
                .create_swapchain(&create_info, None)
                .expect("Could not recreate the swapchain")
        };

        // Update the used swapchain image
        let old_images = std::mem::replace(
            &mut *self.images.lock(),
            Self::get_images_and_views(
                swapchain,
                &self.loader,
                self.format.format,
                adapter,
                device,
            ),
        );

        // Destroy the old image view
        for (_, view) in old_images {
            device.destroy_image_view(view);
        }

        // Destroy the old swapchain
        unsafe {
            self.loader.destroy_swapchain(*self.raw.lock(), None);
        }

        // Updatah
        *self.raw.lock() = swapchain;
    }
}
