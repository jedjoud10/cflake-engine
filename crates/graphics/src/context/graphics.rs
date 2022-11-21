use std::ffi::{CStr, CString};

use super::{Window, FrameRateLimit, WindowSettings};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk::{self, PhysicalDevice, PhysicalDeviceMemoryProperties, PhysicalDeviceFeatures, PhysicalDeviceProperties, DeviceQueueCreateInfo, DeviceCreateInfo}, Entry, Instance,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicSettings {
    pub validation_layers: Vec<CString>,
    pub instance_extensions: Vec<CString>,
    pub logical_device_extensions: Vec<CString>,
    pub frames_in_swapchain: u32,
}

impl Default for GraphicSettings {
    fn default() -> Self {
        Self {
            validation_layers: vec![CString::new(
                "VK_LAYER_KHRONOS_validation".to_owned(),
            )
            .unwrap()],
            instance_extensions: vec![
                DebugUtils::name().to_owned(),
                Surface::name().to_owned(),
            ],
            logical_device_extensions: vec![
                Swapchain::name().to_owned()
            ],
            frames_in_swapchain: 3,
        }
    }
}

// Internal swapchain data wrapper
struct SwapchainWrapper {
    loader: ash::extensions::khr::Swapchain,
    raw: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    command_buffers: Vec<vk::CommandBuffer>,
    image_index: u32,
    rendering_finished_semaphore: vk::Semaphore,
    rendering_finished_fence: vk::Fence,
    image_available_semaphore: vk::Semaphore,
}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
pub struct Graphics {
    // Context related
    pub(crate) entry: Entry,
    pub(crate) instance: Instance,
    debug_utils: DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    // Physical and logical devices
    pub(crate) device: ash::Device,
    pub(crate) physical_device: PhysicalDevice,

    // Physical device properties
    pub(crate) physical_device_memory_properties:
        PhysicalDeviceMemoryProperties,
    pub(crate) physical_device_features: PhysicalDeviceFeatures,
    pub(crate) physical_device_properties: PhysicalDeviceProperties,

    // Graphics queue and present queue
    pub(crate) graphics_family_index: u32,
    pub(crate) present_family_index: u32, 
    
    // Global command buffers
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    pub(crate) command_pool: vk::CommandPool,

    // Window and swapchain
    display_handle: raw_window_handle::RawDisplayHandle,
    window_handle: raw_window_handle::RawWindowHandle,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
    swapchain: Option<SwapchainWrapper>,
}

impl Graphics {
    // Function that will be called per physical device to check if we can use it
    
    
    // Create a new Vulkan graphics context based on the window wrapper
    // This will create the window surface, then pick out a physical device
    // It will then create the swapchain and setup the swapchain images
    pub(crate) unsafe fn new(
        title: &str,
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
        window_settings: &WindowSettings,
    ) -> Graphics {
        // Load the loading functions
        let entry = Entry::load().unwrap();

        // Get a window and display handle to the winit window
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();

        // Create the app info
        let app_name = CString::new(title.to_owned()).unwrap();
        let engine_name = CString::new("cFlake engine").unwrap();
        let app_info = *vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(vk::API_VERSION_1_3)
            .application_version(0)
            .engine_version(0)
            .engine_name(&engine_name);

        // Create the debug messenger create info
        let mut debug_messenger_create_info = super::create_debug_messenger_create_info();

        // Get the required instance extensions from the handle
        let mut extension_names_ptrs =
            ash_window::enumerate_required_extensions(display_handle)
                .unwrap()
                .to_vec();
        extension_names_ptrs.extend(
            graphic_settings
                .instance_extensions
                .iter()
                .map(|s| s.as_ptr()),
        );

        // Get the required validation layers
        let validation_ptrs = graphic_settings
            .validation_layers
            .iter()
            .map(|cstr| cstr.as_ptr())
            .collect::<Vec<_>>();

        // Setup the instance create info
        let instance_create_info = *vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&validation_ptrs)
            .enabled_extension_names(&extension_names_ptrs)
            .push_next(&mut debug_messenger_create_info);

        // Create the instance
        let instance = entry
            .create_instance(&instance_create_info, None)
            .unwrap();

        // Create the debug messenger and the debug utils
        let debug_utils = DebugUtils::new(&entry, &instance);
        let debug_messenger = debug_utils
            .create_debug_utils_messenger(
                &debug_messenger_create_info,
                None,
            )
            .unwrap();

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

        // Get a list of devices that we can use
        let (physical_device, physical_device_features, physical_device_properties) = super::pick_physical_device(&instance, &surface_loader, surface, graphic_settings);

        // Get physical device memory properties
        let physical_device_memory_properties = instance
            .get_physical_device_memory_properties(physical_device);

        // Get the queue family from this physical device
        let queue_families = instance
            .get_physical_device_queue_family_properties(
                physical_device,
            );

        // Find the index for the graphics queue family
        let graphics_family_index = super::pick_graphics_queue(&queue_families, &surface_loader, physical_device, surface, false, vk::QueueFlags::GRAPHICS);
        let present_family_index = super::pick_graphics_queue(&queue_families, &surface_loader, physical_device, surface, true, vk::QueueFlags::GRAPHICS);

        // Specify the logical device's queue info
        let queue_create_info = DeviceQueueCreateInfo::builder()
            .queue_priorities(&[1.0f32])
            .queue_family_index(graphics_family_index);
        let infos = [queue_create_info.build()];

        // Create logical device create info
        let logical_device_extensions = graphic_settings
            .logical_device_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();
        let logical_device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&infos)
            .enabled_extension_names(&logical_device_extensions)
            .enabled_features(&physical_device_features);

        // Create the logical device
        let device = instance
            .create_device(
                physical_device,
                &logical_device_create_info,
                None,
            )
            .expect("Could not create the logical device");

        // Create a command pool for rendering
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder()
                .queue_family_index(graphics_family_index);
        let command_pool = device
            .create_command_pool(&command_pool_create_info, None)
            .unwrap();
        
        // Vector that will store the logical device command buffers
        let mut command_buffers = Vec::<vk::CommandBuffer>::new();

        // Get the supported surface formats khr
        let format = super::pick_surface_format(&surface_loader, physical_device, surface);

        // Create the swapchain image size
        let extent = *vk::Extent2D::builder()
            .height(window.inner_size().height)
            .width(window.inner_size().width);

        // Pick the most appropriate present mode
        let present = super::pick_presentation_mode(&surface_loader, physical_device, surface, window_settings);

        // Create the swap chain create info
        let swapchain_create_info = super::create_swapchain_create_info(surface, format, extent, present);

        // Create the loader and the actual swapchain
        let swapchain_loader = ash::extensions::khr::Swapchain::new(
            &instance,
            &device,
        );
        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Could not create the swapchain");

        // Create the image handles
        let swapchain_images =
            swapchain_loader.get_swapchain_images(swapchain).unwrap();

        // Semaphore that is signaled whenever we have a new available image
        let image_available_semaphore = create_semaphore(&device);

        // Semaphore that is signaled when we finished rendering
        let rendering_finished_semaphore = create_semaphore(&device);

        // Fence that is signaled when we finished rendering
        let rendering_finished_fence_create_info =
            vk::FenceCreateInfo::builder();
        let rendering_finished_fence = device
            .create_fence(&rendering_finished_fence_create_info, None)
            .unwrap();

        // Create a multiple command buffer
        let command_buffer_allocation_info =
            vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(
                    swapchain_images.len() as u32
                )
                .level(vk::CommandBufferLevel::PRIMARY);
        let swapchain_command_buffers = device
            .allocate_command_buffers(&command_buffer_allocation_info)
            .unwrap();
        command_buffers.extend(swapchain_command_buffers.clone());

        // Record each command buffer separately
        for i in 0..swapchain_images.len() {
            // Get command buffer for the correspoding image view
            let command_buffer = swapchain_command_buffers[i];
            let image = swapchain_images[i];

            // Record the command buffer and set the clear frame
            let command_buffer_begin_info =
                vk::CommandBufferBeginInfo::builder().flags(
                    vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                );
            
            // Image subresource range
            let subresource_range =
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1);

            // Reset the presented image layout to be able to clear it
            let present_to_clear = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(graphics_family_index)
                .dst_queue_family_index(graphics_family_index)
                .image(image)
                .subresource_range(*subresource_range);

            // Convert the clear image layout to be able to present it
            let clear_to_present = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(graphics_family_index)
                .dst_queue_family_index(graphics_family_index)
                .image(image)
                .subresource_range(*subresource_range);

            // Start recording the command buffer
            device
                .begin_command_buffer(
                    command_buffer,
                    &command_buffer_begin_info,
                )
                .unwrap();

            // Convert image layouts and wait
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*present_to_clear],
            );


            // Set the clear color of the image view
            let mut clear_color_value =
                vk::ClearColorValue::default();
            clear_color_value.float32 = [1.0; 4];



            // Clear the color of the image
            device.cmd_clear_color_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color_value,
                &[*subresource_range],
            );
            

            // Convert image layouts and wait
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*clear_to_present],
            );

            // Stop recording the command buffer
            device
                .end_command_buffer(command_buffer)
                .unwrap();
        }

        Self {
            entry,
            instance,
            debug_utils,
            debug_messenger,

            device: device,
            physical_device,
            physical_device_memory_properties,
            physical_device_features,
            physical_device_properties,
            
            graphics_family_index,
            present_family_index,
            
            command_pool,
            command_buffers,
        
            display_handle,
            window_handle,
            surface,
            surface_loader,
            swapchain: Some(SwapchainWrapper {
                loader: swapchain_loader,
                raw: swapchain,
                images: swapchain_images,
                command_buffers: swapchain_command_buffers,
                image_index: 0,
                rendering_finished_semaphore,
                rendering_finished_fence,
                image_available_semaphore,
            }),
        }
    }


    // Draw the main window swapchain sheize
    pub(crate) unsafe fn draw(&mut self) {
        if let Some(swapchain) = &mut self.swapchain {
            // Get the next free image and render to it
            let (image_index, _) = swapchain
                .loader
                .acquire_next_image(
                    swapchain.raw,
                    u64::MAX,
                    swapchain.image_available_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();
            swapchain.image_index = image_index;

            // Get the command buffer that is used to render to the current image
            let command_buffer = swapchain.command_buffers
                [image_index as usize];

            // Wait until we have a presentable image we can write to
            let submit_info = *vk::SubmitInfo::builder()
                .wait_semaphores(&[swapchain.image_available_semaphore])
                .signal_semaphores(&[swapchain.rendering_finished_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::TRANSFER])
                .command_buffers(&[command_buffer]);

            // Submit the command buffers
            let queue = self.device
                .get_device_queue(self.graphics_family_index, 0);
            self.device
                .queue_submit(
                    queue,
                    &[submit_info],
                    swapchain.rendering_finished_fence,
                )
                .unwrap();

            // Wait until the command buffers finished executing so we can present the image
            let present_info = *vk::PresentInfoKHR::builder()
                .swapchains(&[swapchain.raw])
                .wait_semaphores(&[swapchain.rendering_finished_semaphore])
                .image_indices(&[image_index]);
            
            // Present the image to the screen
            swapchain
                .loader
                .queue_present(queue, &present_info)
                .unwrap();

            // Wait till the last frame finished rendering
            self.device.wait_for_fences(&[swapchain.rendering_finished_fence], true, u64::MAX).unwrap();
            self.device.reset_fences(&[swapchain.rendering_finished_fence]).unwrap();
        }
    }

    // Destroy the context after we've done using it
    pub(crate) unsafe fn destroy(mut self) {
        self.device.device_wait_idle().unwrap();

        // Destroy swapchain
        let swapchain = self.swapchain.take().unwrap();
        self.device.destroy_semaphore(
            swapchain.image_available_semaphore,
            None,
        );
        self.device.destroy_semaphore(
            swapchain.rendering_finished_semaphore,
            None
        );
        self.device.destroy_fence(
            swapchain.rendering_finished_fence,
            None
        );
        swapchain
            .loader
            .destroy_swapchain(swapchain.raw, None);

        // Destroy device
        self.device.device_wait_idle().unwrap();
        self.device.free_command_buffers(self.command_pool, self.command_buffers.as_slice());
        self.device
            .destroy_command_pool(self.command_pool, None);
        self.device.destroy_device(None);

        // Destroy context
        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils.destroy_debug_utils_messenger(
            self.debug_messenger,
            None,
        );
        self.instance.destroy_instance(None);
    }
}

unsafe fn create_semaphore(device: &ash::Device) -> vk::Semaphore {
    let image_available_semaphore = device
        .create_semaphore(
            &vk::SemaphoreCreateInfo::default(),
            None,
        )
        .unwrap();
    image_available_semaphore
}
