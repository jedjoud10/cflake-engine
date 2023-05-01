use crate::{
    Cached, FrameRateLimit, Graphics, InternalGraphics, StagingPool, Window, WindowSettings,
};

use dashmap::DashMap;
use nohash_hasher::NoHashHasher;
use parking_lot::Mutex;
use std::sync::Arc;
use systemstat::Platform;
use wgpu::RequestAdapterOptions;
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

    // TODO: Try to find a pure rust alternative to SHADERC to compile glsl
    // Don't use naga since it's shit (the glsl interface at least)
    // and NO I AM NOT GOING TO USE WGLSL
    let backends = wgpu::Backends::VULKAN;

    // Create the WGPU instance that will pick an appropriate backend
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler: Default::default(),
    });

    // Create the rendering surface
    let surface = unsafe { instance.create_surface(&window.as_ref()).unwrap() };

    // Modified default limits are sufficient
    let mut limits = wgpu::Limits::default();
    limits.max_push_constant_size = 128;
    limits.max_storage_buffer_binding_size = 128 << 20;

    // Required device features
    let features = wgpu::Features::TEXTURE_COMPRESSION_BC
        | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
        | wgpu::Features::BUFFER_BINDING_ARRAY
        | wgpu::Features::MULTI_DRAW_INDIRECT
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::DEPTH32FLOAT_STENCIL8
        | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::PUSH_CONSTANTS
        | wgpu::Features::SPIRV_SHADER_PASSTHROUGH;

    // Checks if we should use integrated graphics
    fn use_integrated_gpu() -> bool {
        let system = systemstat::System::new();
        !system.on_ac_power().ok().unwrap_or(true)
    }

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

        // Give adapters with higher bind group size, texture size, and allocation size a better score 
        score += limits.max_bind_groups as i32
            + limits.max_texture_dimension_1d as i32
            + limits.max_texture_dimension_2d as i32
            + limits.max_texture_dimension_3d as i32;


        // If we are not connected to AC power, use integrated graphics 
        if use_integrated_gpu() {
            score = i32::MAX;
        }

        (adapter, score)
    }).max_by(|(_, a), (_, b)| i32::cmp(a, b))
    .expect("Did not find a suitable GPU!");

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

    // Get surface data
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities
        .formats
        .contains(&wgpu::TextureFormat::Bgra8Unorm)
        .then_some(wgpu::TextureFormat::Bgra8Unorm)
        .expect("Adapter does not support Bgra8Unorm surface format");

    // Pick the appropriate present mode
    let present_mode = match settings.limit {
        FrameRateLimit::VSync => wgpu::PresentMode::AutoVsync,
        FrameRateLimit::Limited(_) => wgpu::PresentMode::AutoNoVsync,
        FrameRateLimit::Unlimited => wgpu::PresentMode::AutoNoVsync,
    };

    // Create the surface configuration
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    // Create the Graphics context wrapper
    let graphics = Graphics(Arc::new(InternalGraphics {
        instance,
        device,
        adapter,
        queue,
        encoders: thread_local::ThreadLocal::default(),
        staging: StagingPool::new(),
        shaderc: shaderc::Compiler::new().unwrap(),
        cached: Cached {
            shaders: Default::default(),
            samplers: Default::default(),
            pipeline_layouts: Default::default(),
            bind_groups: Default::default(),
            bind_group_layouts: Default::default(),
        },
        acquires: Default::default(),
        submissions: Default::default(),
        stalls: Default::default(),
    }));

    // Create the Window wrapper
    let window = Window {
        settings,
        raw: window,
        size,
        surface,
        surface_config,
        surface_capabilities,
        presentable_texture: None,
        presentable_texture_view: None,
    };

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
