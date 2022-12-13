use crate::{FrameRateLimit, Graphics, Window, WindowSettings, Surface, Adapter, Device, Queue, Swapchain, Instance};
use parking_lot::Mutex;
use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Create the Vulkan context wrapper and a Window wrapper
pub(crate) unsafe fn init_context_and_window(
    app_name: String,
    engine_name: String,
    el: &EventLoop<()>,
    settings: WindowSettings,
) -> (Graphics, Window) {
    // Create a winit window
    let window = init_window(el, &settings);

    // Create the low-level mid wrappers around raw Vulkan objects
    let instance = Instance::new(&window, app_name, engine_name);
    let surface = Surface::new(&instance, &window);
    let adapter = Adapter::pick(&instance, &surface);
    let device = Device::new(&instance, &adapter);
    let queue = Queue::new(&device, &adapter);
    let vsync = matches!(settings.limit, FrameRateLimit::VSync);
    let swapchain = Swapchain::new(
        &adapter, &surface, &device, &instance, &window, vsync,
    );

    // Create the graphics wrapper
    let graphics = super::graphics::Graphics(Arc::new(super::graphics::InternalGraphics {
        instance,
        surface,
        adapter,
        device,
        queue,
        swapchain,
    }));

    // Create the window wrapper
    let window = Window { settings, raw: window };

    (graphics, window)
}

// Init a winit window
fn init_window(
    el: &EventLoop<()>,
    settings: &WindowSettings,
) -> winit::window::Window {
    WindowBuilder::default()
        .with_fullscreen(
            settings
                .fullscreen
                .then_some(Fullscreen::Borderless(None)),
        )
        .with_title(&settings.title)
        .build(&el)
        .unwrap()
}
