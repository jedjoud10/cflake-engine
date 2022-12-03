use std::sync::Arc;
use log_err::{LogErrResult, LogErrOption};
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceExtensions, QueueFamilyProperties, DeviceCreateInfo, QueueCreateInfo, Queue, Features}, VulkanLibrary, swapchain::{Surface, Swapchain, SwapchainCreateInfo, PresentMode}, image::{SwapchainImage, ImageUsage}, memory::allocator::StandardMemoryAllocator, command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}};
use vulkano_win::VkSurfaceBuild;
use winit::{window::{Fullscreen, WindowBuilder}, event_loop::EventLoop};
use crate::{WindowSettings, Window, Graphics, FrameRateLimit};


// Create the Vulkan context wrapper and a Window wrapper
pub(crate) fn init_context_and_window(
    app_name: String,
    engine_name: String,
    el: &EventLoop<()>,
    settings: WindowSettings
) -> (Graphics, Window) {
    // Load the Vulkan instance
    let instance = load_instance(app_name, engine_name);

    // Find an appropriate physical device
    let physical = pick_physical_device(instance.clone());

    // Create the Winit Window and Vulkan surface
    let surface = init_surface(instance.clone(), el, &settings); 

    // Get a graphics and presentable queue
    let queue = find_presentable_queue_family(physical.clone(), surface.clone());

    // Create a logical device
    let (device, queue) = create_logical_device(physical.clone(), queue);

    // Create a rendering swapchain
    let window = surface.object().unwrap().downcast_ref::<winit::window::Window>().unwrap();
    let (swapchain, images) = create_swapchain(
        device.clone(),
        surface.clone(),
        window, physical.clone(),
        &settings
    );

    // Create a memory allocator
    let memory_allocator = create_allocator(&device);    

    // Create a command buffer allocator
    let cmd_buffer_allocator = create_cmd_allocator(&device);

    // Create the graphics wrapper
    let graphics = Graphics {
        instance,
        physical,
        queue,
        device,
        swapchain,
        images,
        memory_allocator,
        cmd_buffer_allocator,
    };

    // Create the window wrapper
    let window = Window {
        settings,
        surface,
    };

    (graphics, window)
}

// Create a memory allocator
fn create_allocator(device: &Arc<Device>) -> Arc<StandardMemoryAllocator> {
    log::debug!("Initializing the standard memory allocator");
    Arc::new(StandardMemoryAllocator::new_default(device.clone()))
}

// Create a command buffer allocator
fn create_cmd_allocator(device: &Arc<Device>) -> Arc<StandardCommandBufferAllocator> {
    log::debug!("Initializing the standard command buffer allocator");

    let create_info = 
        StandardCommandBufferAllocatorCreateInfo::default();

    Arc::new(StandardCommandBufferAllocator::new(device.clone(), create_info))
}

// Init a Vulkan surface (winit window)
fn init_surface(instance: Arc<Instance>, el: &EventLoop<()>, settings: &WindowSettings) -> Arc<Surface> {
    WindowBuilder::default()
        .with_fullscreen(
            settings
            .fullscreen
            .then_some(Fullscreen::Borderless(None)),
        )
        .with_title(&settings.title)
        .build_vk_surface(&el, instance)
        .unwrap()
}

// Create the Vulkan instance
fn load_instance(app_name: String, engine_name: String) -> Arc<Instance> {
    let library = VulkanLibrary::new().log_expect("Cock");
    let required_extensions = vulkano_win::required_extensions(&library);
    let create_info = InstanceCreateInfo {
        application_name: Some(app_name),
        engine_name: Some(engine_name),
        enabled_extensions: required_extensions,
        enumerate_portability: true,
        ..Default::default()
    };
    Instance::new(library, create_info).log_expect("Cock")
}

// List of device extensions that we will use
fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    }
}

// List of device features that we will use
fn device_features() -> Features {
    Features {
        tessellation_shader: true,
        multi_draw: true,
        multi_draw_indirect: true,
        ..Default::default()
    }
}

// Iterate through all physical devices and pick an optimal
fn pick_physical_device(instance: Arc<Instance>) -> Arc<PhysicalDevice> {
    // Find the best GPU that supports the extensions
    let adapter = instance
        .enumerate_physical_devices()
        .log_expect("Could not enumerate physical devices")
        .find(|p| {
            log::debug!("Checking if {} is suitable...", p.properties().device_name);
            
            // Check if the extensions are supported
            let extensions = p.supported_extensions().contains(&device_extensions());
            log::debug!("Supported extensions: {}", extensions);

            // Check if the features are supported
            let features = p.supported_features().contains(&device_features());
            log::debug!("Supported features: {}", features);

            // Check if the device type matches
            let optimal = matches!(p.properties().device_type, PhysicalDeviceType::DiscreteGpu);
            log::debug!("Is device type optimal: {}", optimal);

            features && optimal
        })
        .log_expect("Could not pick a physical device");
    log::debug!("Chose the {} as the physical device", adapter.properties().device_name);
    adapter
}

// Find a graphics and presentable queue family
fn find_presentable_queue_family(physical: Arc<PhysicalDevice>, surface: Arc<Surface>,) -> u32 {
    for family in physical.queue_family_properties() {
        log::debug!("Found a queue family with {:?} queue(s)", family.queue_count);
    }

    // Internal function used by the iterator
    fn check_queue(q: &QueueFamilyProperties, adapter: Arc<PhysicalDevice>, i: usize, surface: Arc<Surface>) -> bool {
        let graphics = q.queue_flags.graphics;
        let presentable = adapter.surface_support(i as u32, &surface).unwrap_or_default();
        graphics && presentable
    }

    physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(i, q)| check_queue(q, physical.clone(), i, surface.clone()))
        .log_expect("couldn't find a graphical queue family") as u32
}

// Create a logical device from a physical device and the main queue family index
fn create_logical_device(physical: Arc<PhysicalDevice>, queue_family_index: u32) -> (Arc<Device>, Arc<Queue>) {
    // Logical device create info where we can also specify extensions
    let info = DeviceCreateInfo {
        queue_create_infos: vec![QueueCreateInfo { queue_family_index, ..Default::default() }],
        enabled_extensions: device_extensions(),
        enabled_features: device_features(),
        ..Default::default()
    };

    //Create the logical device and fetch the queue
    let (device, queues) = 
        Device::new(physical,info)
        .log_expect("Could not create the logical device");
    let queues = queues.collect::<Vec<_>>();
    log::debug!("Created a logical device with {} queue(s)", queues.len());
    return (device, queues[0].clone());
} 

// Create the swapchain that we will present to
fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
    window: &winit::window::Window,
    physical: Arc<PhysicalDevice>,
    window_settings: &WindowSettings,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    // Get surface capabilities
    let capabilites = physical.surface_capabilities(
        &surface,
        Default::default()
    ).unwrap();

    // Get supported image formats
    let formats = physical.surface_formats(
        &surface,
        Default::default()
    ).unwrap();

    // Get supported present modes
    let modes = physical
        .surface_present_modes(&surface)
        .unwrap()
        .collect::<Vec<_>>();

    // Get supported present modes
    let mode = if matches!(window_settings.limit, FrameRateLimit::VSync) {
        if modes.contains(&PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else {
            PresentMode::Fifo
        }
    } else {
        PresentMode::Immediate
    };
    log::debug!("Chose the present mode {:?}", mode);

    // Create the swapchain create info
    let create_info = SwapchainCreateInfo {
        min_image_count: capabilites.min_image_count,
        image_format: Some(formats[0].0),
        image_extent: window.inner_size().into(),
        present_mode: mode,
        image_usage: ImageUsage {
            color_attachment: true,
            ..ImageUsage::empty()
        },
        ..Default::default()
    };

    // Create the swapchain
    Swapchain::new(
        device,
        surface,
        create_info
    ).log_expect("Could not create the swapchain")
}