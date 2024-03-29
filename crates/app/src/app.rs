use graphics::{FrameRateLimit, WindowSettings};
use log::LevelFilter;
use mimalloc::MiMalloc;

use std::sync::mpsc;
use utils::UtilsSettings;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use world::{Event, Init, Shutdown, State, System, Systems, Tick, Update, World};

use crate::systems::gui::EventStatsDurations;

//#[global_allocator]
//static GLOBAL: MiMalloc = MiMalloc;

/// An app that can be built that will use cFlake engine.
/// It uses the builder pattern to set settings and to register custom events
pub struct App {
    // Graphical settings
    window: WindowSettings,

    // Asset and IO
    author_name: String,
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,
    tick_rate: u32,

    // Main app resources
    systems: Systems,
    world: World,
    logging_level: log::LevelFilter,
    el: EventLoop<()>,
}

impl Default for App {
    fn default() -> Self {
        let (world, systems) = world::setup();

        Self {
            window: WindowSettings {
                title: "Default title".to_string(),
                limit: FrameRateLimit::default(),
                fullscreen: false,
            },
            author_name: "cFlake Dev".to_string(),
            app_name: "cFlake Prototype Game".to_string(),
            app_version: 1,
            engine_name: "cFlake Game Engine".to_string(),
            engine_version: 1,
            tick_rate: 128,
            systems,
            el: EventLoop::new(),
            logging_level: log::LevelFilter::Debug,
            world,
        }
    }
}

impl App {
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

    /// Set the author name of the app.
    pub fn set_author_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self
    }

    /// Set the app name (and also the window title).
    pub fn set_app_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self.window.title = name.to_string();
        self
    }

    /// Set the app version.
    pub fn set_app_version(mut self, version: u32) -> Self {
        self.app_version = version;
        self
    }

    /// Set the tick rate 
    pub fn set_tick_rate(mut self, tick_rate: u32) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Insert a new system into the app and register the necessary events.
    pub fn insert_system(mut self, callback: impl FnOnce(&mut System) + 'static) -> Self {
        self.systems.insert(callback);
        self
    }

    /// Insert a single init event that will be called during initialization.
    pub fn insert_init<ID>(self, init: impl Event<Init, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_init(init);
        })
    }

    /// Insert a single update event that will be called every frame.
    pub fn insert_update<ID>(self, update: impl Event<Update, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_update(update);
        })
    }

    /// Insert a single shutdown event that will be called when the engine shuts down.
    pub fn insert_shutdown<ID>(self, shutdown: impl Event<Shutdown, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_shutdown(shutdown);
        })
    }

    /// Insert a single tick event that will execute [`N times`](utils::TICKS_PER_SEC) per second.
    pub fn insert_tick<ID>(self, tick: impl Event<Tick, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_tick(tick);
        })
    }

    /// Insert a single window event that receives Winit window events.
    pub fn insert_window<ID>(self, event: impl Event<WindowEvent<'static>, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_window(event);
        })
    }

    /// Insert a single device event that receives Winit device events.
    pub fn insert_device<ID>(self, event: impl Event<DeviceEvent, ID> + 'static) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_device(event);
        })
    }

    /// Set the logger level that can hide/show log messages.
    pub fn set_level_filter(mut self, level: log::LevelFilter) -> Self {
        self.logging_level = level;
        self
    }

    // Initialize the global logger (also sets the output file)
    fn init_logger(&mut self, sender: mpsc::Sender<String>) {
        use fern::colors::*;

        // File logger with no colors. Will write into the given cache buffer
        fn file_logger(sender: mpsc::Sender<String>) -> fern::Dispatch {
            fern::Dispatch::new()
                .format(move |out, _, record| {
                    out.finish(format_args!(
                        "[{thread_name}][{date}][{target}][{level}] {message}",
                        date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        target = record.target(),
                        level = record.level(),
                        message = record.args(),
                        thread_name = std::thread::current().name().unwrap_or("none")
                    ));
                })
                .chain(sender)
        }

        // Console logger with pwetty colors
        fn console_logger(
            colors_level: ColoredLevelConfig,
            colors_line: ColoredLevelConfig,
        ) -> fern::Dispatch {
            fern::Dispatch::new().format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}[{thread_name}][{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                    ),
                    date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    target = record.target(),
                    level = colors_level.color(record.level()),
                    message = message,
                    thread_name = std::thread::current().name().unwrap_or("none"),
                ));
            }).chain(std::io::stdout())
        }

        // Override logging level with environment variable
        if std::env::var("CFLAKE_LOGGING_TRACE").is_ok() {
            self.logging_level = LevelFilter::Trace;
        }

        // Color config for the line color
        let colors_line = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::White)
            .debug(Color::White)
            .trace(Color::BrightBlack);

        // Color config for the level
        let colors_level = colors_line
            .info(Color::Green)
            .debug(Color::Blue)
            .warn(Color::Yellow)
            .error(Color::Red);

        // Level filter for wgpu and subdependencies
        let wgpu_filter = match self.logging_level {
            log::LevelFilter::Off => log::LevelFilter::Off,
            log::LevelFilter::Debug => log::LevelFilter::Off,
            log::LevelFilter::Trace => log::LevelFilter::Trace,
            _ => log::LevelFilter::Warn,
        };

        fern::Dispatch::new()
            .level_for("wgpu", wgpu_filter)
            .level_for("wgpu_core", wgpu_filter)
            .level_for("wgpu_hal", wgpu_filter)
            .level_for("wgpu_core", wgpu_filter)
            .level(self.logging_level)
            .chain(console_logger(colors_level, colors_line))
            .chain(file_logger(sender))
            .apply()
            .unwrap();
    }

    
    // Internal function to help us add systems
    fn regsys(&mut self, sys: impl FnOnce(&mut System) + 'static) {
        self.systems.insert(sys);
    }

    // Insert the required default systems
    fn insert_default_systems(mut self, receiver: mpsc::Receiver<String>) -> Self {
        // Create the rayon global thread pool
        rayon::ThreadPoolBuilder::new()
            .thread_name(|i| format!("worker-thread-{i}"))
            .build_global()
            .unwrap();

        // Input system
        self.regsys(input::system);

        // Assets system
        self.regsys(assets::system);

        // Scene systems
        self.regsys(ecs::post_frame_or_tick);
        self.regsys(ecs::pre_frame_or_tick);
        self.regsys(ecs::common);

        // Hierarchy system
        self.regsys(coords::hierarchy);

        // World system
        self.regsys(world::system);

        // Utils systems
        self.regsys(utils::time);
        self.regsys(utils::io);
        self.regsys(utils::file_logger);

        // Audio system
        self.regsys(audio::system);

        // Networking system
        self.regsys(networking::system);

        // Graphics systems
        self.regsys(graphics::common);
        self.regsys(graphics::acquire);
        self.regsys(graphics::present);

        // Rendering systems
        self.regsys(rendering::systems::camera::system);
        self.regsys(rendering::systems::composite::system);
        self.regsys(rendering::systems::matrix::system);
        self.regsys(rendering::systems::rendering::system);
        self.regsys(rendering::systems::lights::system);
        self.regsys(rendering::systems::environment::system);

        // Terrain systems
        self.regsys(terrain::systems::manager::system);
        self.regsys(terrain::systems::generation::system);
        self.regsys(terrain::systems::init::system);
        self.regsys(terrain::systems::readback::readback_begin_system);
        self.regsys(terrain::systems::readback::readback_end_system);
        self.regsys(terrain::systems::cull::system);

        // Physics systems
        self.regsys(physics::system);

        // Gui system + stats update event
        self.regsys(gui::common);
        self.regsys(gui::acquire);
        self.regsys(gui::display);

        // Camera system and statistics system
        self.regsys(crate::systems::camera::system);
        self.regsys(crate::systems::gui::system);

        // Fetch names and versions
        let app_name = self.app_name.clone();
        let app_version = self.app_version;
        let engine_name = self.engine_name.clone();
        let engine_version = self.engine_version;
        let author_name = self.author_name.clone();

        // Insert the utils' settings
        self.world.insert(UtilsSettings {
            author_name: author_name.clone(),
            app_name: app_name.clone(),
            log_receiver: Some(receiver),
            tick_rate: self.tick_rate,
            tick_rate_max: 8,
        });

        // Insert the graphics API window resource
        let window_settings = self.window.clone();
        self.world.insert(window_settings);

        // Print app / author / engine data
        log::info!("App Name: '{app_name}', App Version: '{app_version}'");
        log::info!("Engine Name: '{engine_name}', Engine Version: '{engine_version}'");
        log::info!("Author Name: '{author_name}'");
        self
    }

    /// Consume the App builder, and start the engine.
    pub fn execute(mut self) {
        // Enable the environment logger
        let (tx, rx) = mpsc::channel::<String>();
        self.init_logger(tx);

        // Pass the panics to the LOG crate
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            log::error!("{:?}", panic_info.to_string());
            hook(panic_info);
        }));

        // Insert the default systems
        self = self.insert_default_systems(rx);

        // Sort all the stages
        log::debug!("Sorting engine stages...");
        self.systems.init.sort().unwrap();
        self.systems.update.sort().unwrap();
        self.systems.shutdown.sort().unwrap();
        self.systems.window.sort().unwrap();
        self.systems.device.sort().unwrap();
        self.systems.tick.sort().unwrap();

        // Sort & execute the init events
        self.systems.init.execute((&mut self.world, &self.el));

        // Update the EventStatsDurations
        let mut durations = self.world.get_mut::<EventStatsDurations>().unwrap();
        durations.init = self.systems.init.timings().0.to_vec();
        durations.init_total = self.systems.init.timings().1;
        drop(durations);

        // Decompose the app
        let mut world = self.world;
        let el = self.el;
        let mut systems = self.systems;

        // Create the spin sleeper for frame limiting
        let mut sleeper = Self::create_sleeper(self.window.limit);

        // We must now start the game engine (start the winit event loop)
        el.run(move |event, _, cf| match event {
            // Call the window events
            winit::event::Event::WindowEvent {
                window_id: _,
                mut event,
            } => {
                systems.window.execute((&mut world, &mut event));
            }

            // Call the device events
            winit::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                systems.device.execute((&mut world, &event));
            }

            // Call the update events
            winit::event::Event::MainEventsCleared => {
                sleeper.loop_start();

                // Make sure we execute the tick event only 60 times per second
                let time = world.get::<utils::Time>().unwrap();
                let ticks_to_execute = time.ticks_to_execute();
                drop(time);
                if let Some(count) = ticks_to_execute {
                    for _ in 0..count.get() {
                        systems.tick.execute(&mut world);
                    }
                }

                // Execute the update event
                systems.update.execute(&mut world);

                // Execute the tick event 120 times per second
                let time = world.get::<utils::Time>().unwrap();

                // Update "update" and "tick" timings
                if time.frame_count() % 2 == 0 {
                    let mut durations = world.get_mut::<EventStatsDurations>().unwrap();
                    durations.update = systems.update.timings().0.to_vec();
                    durations.update_total = systems.update.timings().1;

                    durations.tick = systems.tick.timings().0.to_vec();
                    durations.tick_total = systems.tick.timings().1;
                    drop(durations);
                }

                // Handle app shutdown
                if let Ok(State::Stopped) = world.get::<State>().map(|x| *x) {
                    *cf = ControlFlow::Exit;
                }

                sleeper.loop_sleep();
            }

            // Call the shutdown events
            winit::event::Event::LoopDestroyed => {
                systems.shutdown.execute(&mut world);
            }
            _ => {}
        });
    }

    // Create a loop sleeper using the given window frame rate limit
    fn create_sleeper(limit: FrameRateLimit) -> spin_sleep::LoopHelper {
        let builder = spin_sleep::LoopHelper::builder();
        let sleeper = if let FrameRateLimit::Limited(limit) = limit {
            builder.build_with_target_rate(limit)
        } else {
            builder.build_without_target_rate()
        };

        match limit {
            FrameRateLimit::Limited(limit) => {
                log::debug!("Created sleeper with a target rate of {limit}")
            }
            FrameRateLimit::VSync => {
                log::debug!("Created sleeper without a target rate (VSync on)")
            }
            FrameRateLimit::Unlimited => {
                log::debug!("Created sleeper without a target rate (VSync off)")
            }
        }
        sleeper
    }
}
