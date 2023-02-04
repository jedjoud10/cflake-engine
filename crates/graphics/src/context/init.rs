use crate::{
    FrameRateLimit, Graphics, InternalGraphics, Window,
    WindowSettings,
};

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
    // Create a winit window and it's wrapper
    let winit_window = Arc::new(init_window(el, &settings));
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(
        winit_window.inner_size(),
    ));
    let window = Window::new(&settings, winit_window.clone(), size);

    // Create the WGPU instance that will pick an appropriate backend
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        dx12_shader_compiler: Default::default(),
    });

    // Create the rendering surface
    let surface = unsafe {
        instance.create_surface(&winit_window.as_ref()).unwrap()
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
    let features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO
        | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
        | wgpu::Features::VERTEX_ATTRIBUTE_64BIT;
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
        .iter()
        .find(|x| x.describe().srgb)
        .unwrap();

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
        format: *surface_format,
        width: window.size().w,
        height: window.size().h,
        present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };

    // Create a new Naga validator
    let flags = naga::valid::ValidationFlags::all();
    let capabilities = naga::valid::Capabilities::all();
    let validator = naga::valid::Validator::new(flags, capabilities);

    // Create a new Naga GLSL parser
    let parser = naga::front::glsl::Parser::default();

    // Create a new Wgpu staging belt
    let staging = wgpu::util::StagingBelt::new(4096);

    // Create the raw graphics context
    let graphics = InternalGraphics {
        surface,
        device,
        queue,
        parser: Mutex::new(parser),
        validator: Mutex::new(validator),
        staging: Mutex::new(staging),
        surface_capabilities,
        surface_config,
        window: winit_window.clone(),
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
