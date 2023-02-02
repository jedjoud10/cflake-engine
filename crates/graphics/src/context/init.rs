use crate::{
    FrameRateLimit, Graphics, Window, WindowSettings, InternalGraphics,
};

use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Temporary resource that the app creates so we can pass data to the graphic system
pub struct GraphicsInit {
    pub window_settings: WindowSettings,
    pub app_name: String,
    pub app_version: u32,
    pub engine_name: String,
    pub engine_version: u32,
}

// Create the Vulkan context wrapper and a Window wrapper
pub(crate) unsafe fn init_context_and_window(
    init: GraphicsInit,
    el: &EventLoop<()>,
) -> (Graphics, Window) {
    let GraphicsInit {
        window_settings,
        app_name,
        app_version,
        engine_name,
        engine_version,
    } = init;

    // Create a winit window and it's wrapper
    let window = init_window(el, &window_settings);
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(
        window.inner_size(),
    ));
    let window = Window::new(window_settings, window, size);

    // Create the WGPU instance that will pick an appropriate backend
    let instance = wgpu::Instance::new(
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        }
    );

    // Create the rendering surface
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    // Pick an appropriate adapter
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }
    )).unwrap();

    // Create a device for the adapter
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None
    )).unwrap();

    // Get surface data
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities.formats.iter().find(|x| x.describe().srgb).unwrap();
    
    // Pick the appropriate present mode
    let present_mode = match window_settings.limit {
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

    // Create the raw graphics context
    let graphics = InternalGraphics {
        surface,
        device,
        queue,
        surface_capabilities,
        surface_config,
    };
    let graphics = Graphics(Arc::new(graphics));

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
