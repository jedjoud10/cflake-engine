use super::{FrameRateLimit, Window, WindowSettings};
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
use bytemuck::{Pod, Zeroable};
use parking_lot::Mutex;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{
    ffi::{CStr, CString},
    sync::Arc,
};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicSettings {
    pub validation_layers: Vec<CString>,
    pub instance_extensions: Vec<CString>,
    pub logical_device_extensions: Vec<CString>,
}

impl Default for GraphicSettings {
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            validation_layers: vec![CString::new(
                "VK_LAYER_KHRONOS_validation".to_owned(),
            ).unwrap()],
            #[cfg(not(debug_assertions))]
            validation_layers: vec![],
            instance_extensions: vec![
                DebugUtils::name().to_owned(),
                Surface::name().to_owned(),
            ],
            logical_device_extensions: vec![
                Swapchain::name().to_owned()
            ],
        }
    }
}

// Internal graphics context that will be shared with other threads
pub(crate) struct InternalGraphics {
    instance: super::Instance,
    adapter: super::Adapter,
    device: super::Device,
    queues: super::Queues,
    surface: super::Surface,
    swapchain: super::Swapchain,
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(Arc<InternalGraphics>);

impl Graphics {
    // Create a new Vulkan graphics context based on the window wrapper
    // This will create the window surface, then pick out a physical device
    // It will then create the swapchain and setup the swapchain images
    pub(crate) unsafe fn new(
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
        window_settings: &WindowSettings,
    ) -> Graphics {
        // Create the Vulkan entry and instance
        let instance = super::create_instance(window, graphic_settings, window_settings);

        // Create a surface from the KHR extension
        let surface = super::create_surface(&instance);

        // Pick a physical device (adapter)
        let adapter = super::pick_adapter(&instance, &surface, graphic_settings);

        // Create the queues that we will instantiate
        let mut queues = super::create_queues(&adapter, &surface, &instance);

        // Create a new device with those queues
        let device = super::create_device(&instance, &adapter, &mut queues, graphic_settings);

        // Create a swapchain we can render to
        let swapchain = super::create_swapchain(&adapter, &surface, &device, &instance, window, window_settings);

        Self(Arc::new(InternalGraphics {
            instance,
            adapter,
            device,
            queues,
            surface,
            swapchain,
        }))
    }

    // Get the instance
    // Get the adapter
    pub(crate) fn adapter(&self) -> &super::Adapter {
        &self.0.adapter
    }

    // Get the device
    pub(crate) fn device(&self) -> &super::Device {
        &self.0.device
    }

    // Get the queues
    pub(crate) fn queues(&self) -> &super::Queues {
        &self.0.queues
    }

    // Get the surface
    pub(crate) fn surface(&self) -> &super::Surface {
        &self.0.surface
    }
    
    // Get the swapchain
    pub(crate) fn swapchain(&self) -> &super::Swapchain {
        &self.0.swapchain
    }

    // Draw the main window swapchain sheize
    pub(crate) unsafe fn draw(&mut self, value: f32) {
        /*
        // Get the next free image and render to it
        let (image_index, _) = self.swapchain
            .loader
            .acquire_next_image(
                self.swapchain.raw,
                u64::MAX,
                self.swapchain.image_available_semaphore,
                vk::Fence::null(),
            )
            .unwrap();

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
            .loader
            .queue_present(queue, &present_info)
            .unwrap();

        // Wait till the last frame finished rendering
        self.device
            .wait_for_fences(
                &[swapchain.rendering_finished_fence],
                true,
                u64::MAX,
            )
            .unwrap();
        self.device
            .reset_fences(&[swapchain.rendering_finished_fence])
            .unwrap();
        */
    }

    // Destroy the context after we've done using it
    pub(crate) unsafe fn destroy(mut self) {
        /*
        self.swapchain.destroy(&self.device);
        self.device.destroy();
        self.surface.destroy();
        self.instance.destroy();
        */
    }
}
