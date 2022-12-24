use crate::{
    Adapter, Device, FrameRateLimit, Graphics, Instance, Queue,
    Surface, Swapchain, Window, WindowSettings,
};
use parking_lot::Mutex;
use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Create the Vulkan context wrapper and a Window wrapper
pub(crate) unsafe fn init_context_and_window(
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,
    el: &EventLoop<()>,
    settings: WindowSettings,
) -> (Graphics, Window) {
    // Create a winit window
    let window = init_window(el, &settings);

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
    let vsync = matches!(settings.limit, FrameRateLimit::VSync);
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
    let window = Window {
        settings,
        size,
        raw: window,
    };

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
