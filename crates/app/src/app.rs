use std::sync::mpsc;

use mimalloc::MiMalloc;
use winit::{event_loop::EventLoop, window::Window};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// An app that can be built that will use cFlake engine.
/// It uses the builder pattern to set settings and to register custom events
#[derive(Default)]
pub struct App {}



impl App {
    /// Consume the App builder, and start the engine.
    pub fn execute(self) {
        let (sender, _receiver) = mpsc::channel::<String>();
        crate::logger::init_logger(log::LevelFilter::Debug, sender);
        let el = EventLoop::new().unwrap();
        let window = Window::new(&el).unwrap();

        el.run(move |event, _, _cf| match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event: _,
            } => {}

            winit::event::Event::DeviceEvent {
                device_id: _,
                event: _,
            } => {}

            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }

            winit::event::Event::RedrawRequested(_id) => {}

            _ => {}
        })
        .unwrap();
    }
}
