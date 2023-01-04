use crate::{
    Adapter, Device, FrameRateLimit, Graphics, Instance, Queue,
    Surface, Swapchain, Window, WindowSettings,
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

    // Create a winit window
    let window = init_window(el, &window_settings);

    // Create the low-level mid wrappers around raw Vulkan objects
    let instance = Instance::new(
        &window,
        app_name,
        app_version,
        engine_name,
        engine_version,
    );
    let surface = Surface::new(&instance, &window);
    let adapter = Adapter::pick(&instance, &surface);
    let device = Device::new(&instance, &adapter);
    let queue = Queue::new(&device, &adapter);
    let vsync =
        matches!(window_settings.limit, FrameRateLimit::VSync);
    let swapchain = Swapchain::new(
        &adapter, &surface, &device, &instance, &window, vsync,
    );

    // Create the graphics wrapper
    let graphics = super::graphics::Graphics(Arc::new(
        super::graphics::InternalGraphics {
            instance,
            surface,
            adapter,
            device,
            queue,
            swapchain,
        },
    ));

    // Create the window wrapper
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(
        window.inner_size(),
    ));
    let window = Window::new(
        window_settings,
        window,
        size
    );

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
