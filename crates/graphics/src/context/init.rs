use crate::{FrameRateLimit, Graphics, Window, WindowSettings};
use parking_lot::Mutex;
use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Create the Vulkan context wrapper and a Window wrapper
pub(crate) fn init_context_and_window(
    app_name: String,
    engine_name: String,
    el: &EventLoop<()>,
    settings: WindowSettings,
) -> (Graphics, Window) {
    // Create the graphics wrapper
    let graphics = todo!();

    // Create a winit window
    let raw = init_window(el, &settings);

    // Create the window wrapper
    let window = Window { settings, raw };

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
