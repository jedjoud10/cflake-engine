use graphics::context::{FrameRateLimit, WindowSettings};
use log::LevelFilter;
use mimalloc::MiMalloc;

use std::sync::mpsc;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop}, error::EventLoopError,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// An Engine that can be built that will use cFlake engine.
/// It uses the builder pattern to set settings and to register custom events
pub struct App {
    window: WindowSettings,
    logging_level: log::LevelFilter,
    el: EventLoop<()>,
}

impl App {
    /// Create a new app with default parameters
    pub fn new() -> Result<Self, EventLoopError> {
        Ok(Self {
            window: WindowSettings {
                title: "Default title".to_string(),
                limit: FrameRateLimit::default(),
                fullscreen: false,
            },
            el: EventLoop::new()?,
            logging_level: log::LevelFilter::Debug,
        })
    }

    /// Set the window framerate limit.
    pub fn set_frame_rate_limit(mut self, limit: FrameRateLimit) -> Self {
        self.window.limit = limit;
        self
    }

    /// Set window fullscreen mode.
    pub fn set_window_fullscreen(mut self, toggled: bool) -> Self {
        self.window.fullscreen = toggled;
        self
    }

    /// Consume the App builder, and start the engine.
    pub fn execute(mut self) -> Result<(), EventLoopError> {
        // Enable the environment logger
        let (tx, rx) = mpsc::channel::<String>();
        super::app_utils::init_logger(self.logging_level, tx);
        let el = self.el;
        let mut sleeper = super::app_utils::init_spin_sleeper(self.window.limit);

        let (window, mut graphics) = graphics::context::initialize_phobos_context(&el);

        el.run(move |event, _, cf| match event {
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

            winit::event::Event::RedrawRequested(_id) => {
                sleeper.loop_start();

                sleeper.loop_sleep();
            }

            winit::event::Event::LoopExiting => {}

            _ => {}
        })?;

        Ok(())
    }
}
