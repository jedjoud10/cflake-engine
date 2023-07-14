use crate::{
    FrameRateLimit, Graphics, Window, WindowSettings, InternalGraphics,
};

use dashmap::DashMap;
use nohash_hasher::NoHashHasher;
use parking_lot::Mutex;
use phobos::{AppBuilder, QueueRequest, QueueType, WindowedContext, ContextInit, Swapchain, GPUFeatures, vk};
use std::{sync::Arc, ffi::CStr};
use systemstat::Platform;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Create the Vulkan context wrapper and a Window wrapper
pub(crate) unsafe fn init_context_and_window(
    settings: WindowSettings,
    el: &EventLoop<()>,
) -> (Graphics, Window) {
    // Create a winit window (but don't make it's wrapper)
    let window = Arc::new(init_window(el, &settings));
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(window.inner_size()));        

    /*


    // Pick the appropriate adapter with the supported features and limits
    let (adapter, _) = instance.enumerate_adapters(backends).filter(|adapter| {
        log::debug!("Checking adapter '{}'...", adapter.get_info().name);
        let limits_supported = limits.check_limits(&adapter.limits());
        let features_supported = adapter.features().contains(features);
        let surface_supported = adapter.is_surface_supported(&surface);
        log::debug!("Limits supported: {limits_supported}, features supported: {features_supported}, surface supported: {surface_supported}");
        limits_supported && features_supported
    }).map(|adapter| {
        let limits = adapter.limits();
        let info = adapter.get_info();
        let mut score = 0i32;

        // Dedicated GPU are much better, so favor them when possible
        score += match info.device_type {
            wgpu::DeviceType::DiscreteGpu => 10000,
            _ => -10000,
        };

        // If we are not connected to AC power, use integrated graphics 
        if use_integrated_gpu() {
            score = i32::MAX;
        }

        (adapter, score)
    }).max_by(|(_, a), (_, b)| i32::cmp(a, b))
    .expect("Did not find a suitable GPU!");
    */

    /*
    // Print details about the chosen adapter
    let info = adapter.get_info();
    let name = info.name;
    let backend = info.backend;
    log::debug!("Chosen Adapter: '{name}', Backend: {backend:?} ");

    // Create a device for the adapter
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features,
            limits,
        },
        None,
    ))
    .unwrap();
    */

    // Pick the appropriate present mode
    let present_mode = match settings.limit {
        FrameRateLimit::VSync => phobos::vk::PresentModeKHR::FIFO,
        FrameRateLimit::Limited(_) | FrameRateLimit::Unlimited => phobos::vk::PresentModeKHR::IMMEDIATE,
    };
    
    // Create the phobos context settings
    let phobos = AppBuilder::<'_, winit::window::Window>::new()
        .version((1, 0, 0))
        .name("cFlake engine")
        .validation(true)
        .window(&window)
        .present_mode(present_mode)
        .scratch_size(1 * 1024u64) // 1 KiB scratch memory per buffer type per frame
        .gpu_features(GPUFeatures {
            queues: vec![
                QueueRequest { dedicated: false, queue_type: QueueType::Graphics },
                QueueRequest { dedicated: true, queue_type: QueueType::Transfer },
                QueueRequest { dedicated: true, queue_type: QueueType::Compute }
            ],
            features: vk::PhysicalDeviceFeatures::builder()
                /*
                .robust_buffer_access(true)
                .multi_draw_indirect(true)
                .sampler_anisotropy(true)
                .texture_compression_astc_ldr(true)
                .texture_compression_bc(true)
                .texture_compression_etc2(true)
                .shader_uniform_buffer_array_dynamic_indexing(true)
                .shader_sampled_image_array_dynamic_indexing(true)
                .shader_storage_buffer_array_dynamic_indexing(true)
                .shader_storage_image_array_dynamic_indexing(true)
                .shader_int16(true)
                .shader_int64(true)
                .shader_int16(true)
                */
                .build(),
            features_1_1: vk::PhysicalDeviceVulkan11Features::builder().build(),
            features_1_2: vk::PhysicalDeviceVulkan12Features::builder().build(),
            features_1_3: vk::PhysicalDeviceVulkan13Features::builder().build(),
            device_extensions: vec![],
        })
        .gpu_selector(|mut physical_devices| {
            // Checks if we should use integrated graphics
            fn use_integrated_gpu() -> bool {
                let system = systemstat::System::new();
            
                // Simple check in case we want to disable integrated GPU
                if std::env::var("CFLAKE_DISCARD_INTEGRATED").is_ok() {
                    return false;
                }
            
                !system.on_ac_power().ok().unwrap_or(true)
            }

            physical_devices.pop().unwrap()
            /*
            let index = 0;
            
            for device in physical_devices {
                let name = unsafe { CStr::from_ptr(x.properties().device_name.as_ptr()) };
                let name = name.to_str().unwrap();
                
                log::info!("{}", name);
            };
            */
        })
        .build();
    
    // Actually create the context
    let (
        instance,
        physical_device,
        surface,
        device,
        allocator,
        pool,
        exec,
        frame,
        Some(debug_messenger)
    ) = WindowedContext::init(&phobos).unwrap() else {
        panic!("Asked for debug messenger but didn't get one.")
    };

    // Create the Window wrapper
    let window = Window {
        settings,
        raw: window,
        size,
    };

    // Create the Graphics wrapper
    let internal = InternalGraphics {
        instance,
        physical_device,
        surface,
        device,
        allocator,
        exec,
        frame,
        pool,
        debug_messenger,
    };
    let graphics = Graphics(Arc::new(internal));

    (graphics, window)
}

// Init a winit window
fn init_window(el: &EventLoop<()>, window_settings: &WindowSettings) -> winit::window::Window {
    WindowBuilder::default()
        .with_fullscreen(
            window_settings
                .fullscreen
                .then_some(Fullscreen::Borderless(None)),
        )
        .with_title(&window_settings.title)
        .build(el)
        .unwrap()
}
