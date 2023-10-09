use graphics::context::{FrameRateLimit, WindowSettings};
use log::LevelFilter;
use mimalloc::MiMalloc;
use world::{system::{Registries, System}, prelude::{Event, Shutdown, Tick, Init, Update, Plugin}, world::World};

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
    registries: Registries,
}

impl App {
    /// Create a new [App] with default parameters
    pub fn new() -> Self {
        Self {
            window: WindowSettings {
                title: "Default title".to_string(),
                limit: FrameRateLimit::default(),
                fullscreen: false,
            },
            registries: Registries::default(),
            logging_level: log::LevelFilter::Debug,
        }
    }

    /// Set the initial window settings that we will pass to [winit].
    pub fn set_window_settings(mut self, settings: WindowSettings) -> Self {
        self.window = settings;
        self
    }

    /// Set the logging level for the [log] crate.
    pub fn set_logging_level(mut self, level: log::LevelFilter) -> Self {
        self.logging_level = level;
        self
    }

    /// Insert a plugin with it's registry callback
    pub fn insert_plugin(mut self, plugin: impl Plugin) -> Self {
        plugin.register(&mut self.registries);
        self
    }

    /// Insert a single init system that will be called during initialization.
    pub fn insert_init(mut self, init: impl System<Init>) -> Self {
        self.registries.init.insert(init);
        self
    }

    /// Insert a single update system that will be called every frame.
    pub fn insert_update(mut self, update: impl System<Update>) -> Self {
        self.registries.update.insert(update);
        self
    }

    /// Insert a single shutdown system that will be called when the engine shuts down.
    pub fn insert_shutdown(mut self, shutdown: impl System<Shutdown>) -> Self {
        self.registries.shutdown.insert(shutdown);
        self
    }

    /// Insert a single tick system that will execute [`N times`](utils::TICKS_PER_SEC) per second.
    pub fn insert_tick(mut self, tick: impl System<Tick>) -> Self {
        self.registries.tick.insert(tick);
        self
    }

    /// Insert a single window system that receives [winit::event::WindowEvent].
    pub fn insert_window(mut self, system: impl System<WindowEvent>) -> Self {
        self.registries.window_event.insert(system);
        self
    }

    /// Insert a single device system that receives [winit::event::DeviceEvent].
    pub fn insert_device(mut self, system: impl System<DeviceEvent>) -> Self {
        self.registries.device_event.insert(system);
        self
    }

    /// Consume the App builder, and start the engine.
    pub fn execute(mut self) -> Result<(), EventLoopError> {
        let (tx, rx) = mpsc::channel::<String>();
        super::app_utils::init_logger(self.logging_level, tx);
        let mut sleeper = super::app_utils::init_spin_sleeper(self.window.limit);

        // Create a world and add the EventLoop for it in case plugins wish to use it
        let mut world = World::default();
        world.insert(EventLoop::<()>::new()?);
        world.insert(self.window);

        // Register main plugins
        graphics::plugin::plugin(&mut self.registries);
        input::plugin::plugin(&mut self.registries);
        assets::plugin::plugin(&mut self.registries);

        // Sort the registries
        self.registries.init.sort().unwrap();
        self.registries.update.sort().unwrap();
        self.registries.shutdown.sort().unwrap();
        self.registries.tick.sort().unwrap();
        self.registries.device_event.sort().unwrap();
        self.registries.window_event.sort().unwrap();
        
        // Execute the init registry
        self.registries.init.execute(&mut world, &Init);

        // Begin the winit event loop
        let el = world.remove::<EventLoop<()>>().unwrap();
        el.run(move |event, target, cf| match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => self.registries.window_event.execute(&mut world, &event),

            winit::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => self.registries.device_event.execute(&mut world, &event),

            winit::event::Event::AboutToWait => {
                sleeper.loop_start();
                self.registries.update.execute(&mut world, &Update);
                sleeper.loop_sleep();
            }

            winit::event::Event::LoopExiting => self.registries.shutdown.execute(&mut world, &Shutdown),

            _ => {}
        })?;

        Ok(())
    }
}
