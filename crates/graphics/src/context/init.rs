use crate::{
    FrameRateLimit, Graphics, InternalGraphics, Window,
    WindowSettings,
};

use dashmap::DashMap;
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
        backends: wgpu::Backends::VULKAN,
        dx12_shader_compiler: Default::default(),
    });

    // Create the rendering surface
    let surface = unsafe {
        instance.create_surface(&window.as_ref()).unwrap()
    };

    // Pick an appropriate adapter
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        },
    ))
    .unwrap();

    // Features and limits
    // TODO: Figure out how to remove SPIRV_SHADER_PASSTHROUGH to be able to make this work on Metal
    let features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO
        | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
        | wgpu::Features::POLYGON_MODE_LINE;
    let limits = wgpu::Limits::default();

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

    // Create a new Naga validator
    let flags = naga::valid::ValidationFlags::all();
    let capabilities = naga::valid::Capabilities::all();
    let validator = naga::valid::Validator::new(flags, capabilities);

    // Create a new Naga GLSL parser
    let parser = naga::front::glsl::Parser::default();

    // Create a new Wgpu staging belt
    let staging = wgpu::util::StagingBelt::new(4096);

    // Create the Graphics context wrapper
    let graphics = Graphics(Arc::new(InternalGraphics {
        device,
        queue,
        adapter,
        parser: Mutex::new(parser),
        validator: Mutex::new(validator),
        staging: Mutex::new(staging),
        samplers: DashMap::default(),
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
