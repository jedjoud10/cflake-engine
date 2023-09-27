use std::sync::mpsc;

use mimalloc::MiMalloc;
use winit::{event_loop::EventLoop, window::Window};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// An app that can be built that will use cFlake engine.
/// It uses the builder pattern to set settings and to register custom events
pub struct App {
}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}

impl App {
    /// Consume the App builder, and start the engine.
    pub fn execute(mut self) {
        let (sender, receiver) = mpsc::channel::<String>();
        crate::logger::init_logger(log::LevelFilter::Debug, sender);
        let mut el = EventLoop::new().unwrap();
        let window = Window::new(&el).unwrap();

        el.run(move |event, _, cf| match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                mut event,
            } => {
            }

            winit::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => {
            }

            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }

            winit::event::Event::RedrawRequested(id) => {
            }

            _ => {}
        }).unwrap();
    }
}
