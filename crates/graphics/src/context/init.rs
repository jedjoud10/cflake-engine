use std::sync::Arc;
use log_err::{LogErrResult, LogErrOption};
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceExtensions, QueueFamilyProperties, DeviceCreateInfo, QueueCreateInfo, Queue}, VulkanLibrary, swapchain::{Surface, Swapchain, SwapchainCreateInfo}, image::SwapchainImage};
use vulkano_win::VkSurfaceBuild;
use winit::{window::{Fullscreen, WindowBuilder}, event_loop::EventLoop};
use crate::{WindowSettings, Window, Graphics};


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
    let (swapchain, images) = create_swapchain(device.clone(), surface.clone(), &settings);

    // Create the graphics wrapper
    let graphics = Graphics {
        instance,
        physical,
        queue,
        device,
        swapchain,
        images,
    };

    // Create the window wrapper
    let window = Window {
        settings,
        surface,
    };

    (graphics, window)
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

// Iterate through all physical devices and pick an optimal
fn pick_physical_device(instance: Arc<Instance>) -> Arc<PhysicalDevice> {
    // Device extensions that we *must* support
    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    // Find the best GPU that supports the extensions
    let adapter = instance
        .enumerate_physical_devices()
        .log_expect("Could not enumerate physical devices")
        .find(|p| {
            log::debug!("Checking if {} is suitable...", p.properties().device_name);
            
            // Check if the extensions are supported
            let supported = p.supported_extensions().contains(&extensions);
            log::debug!("Supported extensions: {}", supported);

            // Check if the device type matches
            let optimal = matches!(p.properties().device_type, PhysicalDeviceType::DiscreteGpu);
            log::debug!("Is device type optimal: {}", optimal);

            supported && optimal
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
    window_settings: &WindowSettings,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
       

    // Create the swapchain create info
    let create_info = SwapchainCreateInfo {
        min_image_count: todo!(),
        image_format: todo!(),
        image_color_space: todo!(),
        image_extent: todo!(),
        image_array_layers: todo!(),
        image_usage: todo!(),
        image_sharing: todo!(),
        pre_transform: todo!(),
        composite_alpha: todo!(),
        present_mode: todo!(),
        clipped: todo!(),
        full_screen_exclusive: todo!(),
        win32_monitor: todo!(),
        _ne: todo!(),
    };

    // Create the swapchain
    Swapchain::new(
        device,
        surface,
        create_info
    ).log_expect("Could not create the swapchain")
}