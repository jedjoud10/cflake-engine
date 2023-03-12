use crate::{
    Cached, FrameRateLimit, Graphics, InternalGraphics, StagingPool,
    Window, WindowSettings,
};

use dashmap::DashMap;
use nohash_hasher::NoHashHasher;
use parking_lot::Mutex;
use std::sync::Arc;
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
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(
        window.inner_size(),
    ));

    // Create the WGPU instance that will pick an appropriate backend
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: Default::default(),
    });

    // Create the rendering surface
    let surface =
        unsafe { instance.create_surface(&window.as_ref()).unwrap() };

    // Pick an appropriate adapter
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        },
    ))
    .unwrap();

    // Print details about the chosen adapter
    let info = adapter.get_info();
    let name = info.name;
    let backend = info.backend;
    log::debug!("Chosen Adapter: '{name}', Backend: {backend:?} ");

    // Print details about adapter features & limits
    let limits = adapter.limits();
    let w = limits.max_texture_dimension_1d;
    let h = limits.max_texture_dimension_2d;
    let d = limits.max_texture_dimension_3d;
    log::debug!(
        "Adapter Limits: Max Texture Dimensions: {w}x{h}x{d}"
    );
    log::debug!(
        "Adapter Limits: Max bind groups: {}",
        limits.max_bind_groups
    );
    log::debug!(
        "Adapter Limits: Max bindings per group: {}",
        limits.max_bindings_per_bind_group
    );
    log::debug!(
        "Adapter Limits: Max Push Constants Size: {}",
        limits.max_push_constant_size
    );

    // Required device features
    let features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::PUSH_CONSTANTS;

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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST,
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
        device,
        queue,
        adapter,
        staging: StagingPool::new(),
        cached: Cached {
            samplers: Default::default(),
            pipeline_layouts: Default::default(),
            bind_groups: Default::default(),
            bind_group_layouts: Default::default(),
        },
        shaderc: shaderc::Compiler::new().unwrap(),
        encoders: thread_local::ThreadLocal::default(),
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
fn init_window(
    el: &EventLoop<()>,
    window_settings: &WindowSettings,
) -> winit::window::Window {
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
