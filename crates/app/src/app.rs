use graphics::{FrameRateLimit, WindowSettings};
use mimalloc::MiMalloc;
use std::path::PathBuf;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use world::{
    Event, Init, Shutdown, State, System, Systems, Tick, Update,
    World,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Graphical settings
    window: WindowSettings,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,
    author_name: String,
    app_name: String,
    app_version: u32,
    engine_name: String,
    engine_version: u32,

    // Main app resources
    systems: Systems,
    world: World,
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
            user_assets_folder: None,
            systems,
            el: EventLoop::new(),
            world,
        }
    }
}

impl App {
    // Set the window framerate limit
    pub fn set_frame_rate_limit(
        mut self,
        limit: FrameRateLimit,
    ) -> Self {
        self.window.limit = limit;
        self
    }

    // Set window fullscreen mode
    pub fn set_window_fullscreen(mut self, toggled: bool) -> Self {
        self.window.fullscreen = toggled;
        self
    }

    // Set the assets folder for the user defined assets
    pub fn set_user_assets_path(
        mut self,
        path: impl TryInto<PathBuf>,
    ) -> Self {
        self.user_assets_folder = Some(
            path.try_into()
                .ok()
                .expect("Input path failed to convert into PathBuf"),
        );
        self
    }

    // Set the author name of the app
    pub fn set_author_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self
    }

    // Set the app name (and also the window title)
    pub fn set_app_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self.window.title = name.to_string();
        self
    }

    // Set the app version
    pub fn set_app_version(mut self, version: u32) -> Self {
        self.app_version = version;
        self
    }

    // Insert a new system into the app and register the necessary events
    pub fn insert_system(
        mut self,
        callback: impl FnOnce(&mut System) + 'static,
    ) -> Self {
        self.systems.insert(callback);
        self
    }

    // Insert a single init event
    pub fn insert_init<ID>(
        self,
        init: impl Event<Init, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_init(init);
        })
    }

    // Insert a single update event
    pub fn insert_update<ID>(
        self,
        update: impl Event<Update, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_update(update);
        })
    }

    // Insert a single shutdown event
    pub fn insert_shutdown<ID>(
        self,
        shutdown: impl Event<Shutdown, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_shutdown(shutdown);
        })
    }

    // Insert a single tick event
    pub fn insert_tick<ID>(
        self,
        tick: impl Event<Tick, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_tick(tick);
        })
    }

    // Insert a single window event
    pub fn insert_window<ID>(
        self,
        event: impl Event<WindowEvent<'static>, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_window(event);
        })
    }

    // Insert a single device event
    pub fn insert_device<ID>(
        self,
        event: impl Event<DeviceEvent, ID> + 'static,
    ) -> Self {
        self.insert_system(move |system: &mut System| {
            system.insert_device(event);
        })
    }

    // Initialize the global logger (also sets the output file)
    fn init_logger(&mut self) {
        use fern::colors::Color;
        self.world.insert(utils::ThreadPool::default());
        
        // Color config for the line color
        let colors_line = fern::colors::ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::White)
            .debug(Color::White)
            .trace(Color::BrightBlack);

        // Color config for the level
        let colors_level = colors_line.clone()
            .info(Color::Green)
            .debug(Color::Blue)
            .warn(Color::Yellow)
            .error(Color::Red);

        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                    ),
                    date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    target = record.target(),
                    level = colors_level.color(record.level()),
                    message = message,
                ));
            })
            //.level_for("naga", log::LevelFilter::Warn)
            .level_for("wgpu", log::LevelFilter::Warn)
            .level_for("wgpu_core", log::LevelFilter::Warn)
            .level_for("wgpu_hal", log::LevelFilter::Warn)
            .level(log::LevelFilter::Debug)
            .chain(std::io::stdout())
            .apply().unwrap();
    }

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Enable the environment logger
        self.init_logger();

        // Pass the panics to the LOG crate
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            log::error!("{:?}", panic_info.to_string());
            hook(panic_info);
        }));

        // Insert the default systems
        self = self.insert_default_systems();

        // Sort all the stages
        log::debug!("Sorting engine stages...");
        self.systems.init.sort().unwrap();
        self.systems.update.sort().unwrap();
        self.systems.shutdown.sort().unwrap();
        self.systems.window.sort().unwrap();
        self.systems.device.sort().unwrap();

        // Sort & execute the init events
        self.systems.init.execute((&mut self.world, &self.el));

        // Decompose the app
        let mut world = self.world;
        let el = self.el;
        let mut systems = self.systems;

        // Create the spin sleeper for frame limiting
        let mut sleeper = Self::create_sleeper(self.window.limit);

        // We must now start the game engine (start the winit event loop)
        el.run(move |event, _, cf| match event {
            // Call the update events
            winit::event::Event::MainEventsCleared => {
                sleeper.loop_start();

                // Execute the update event
                systems.update.execute(&mut world);

                // Execute the tick event 120 times per second
                let time = world.get::<utils::Time>().unwrap();

                // Make sure we execute the tick event only 120 times per second
                if let Some(count) = time.ticks_to_execute() {
                    drop(time);

                    // Execute the tick events
                    for _ in 0..count.get() {
                        systems.tick.execute(&mut world);
                    }
                }

                // Handle app shutdown
                if let Ok(State::Stopped) =
                    world.get::<State>().map(|x| *x)
                {
                    *cf = ControlFlow::Exit;
                }

                sleeper.loop_sleep();
            }

            // Call the shutdown events
            winit::event::Event::LoopDestroyed => {
                systems.shutdown.execute(&mut world);
            }

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
            _ => {}
        });
    }

    // Create a loop sleeper using the given window frame rate limit
    fn create_sleeper(
        limit: FrameRateLimit,
    ) -> spin_sleep::LoopHelper {
        let builder = spin_sleep::LoopHelper::builder();
        let sleeper = if let FrameRateLimit::Limited(limit) = limit {
            builder.build_with_target_rate(limit)
        } else {
            builder.build_without_target_rate()
        };

        match limit {
            FrameRateLimit::Limited(limit) => log::debug!(
                "Created sleeper with a target rate of {limit}"
            ),
            FrameRateLimit::VSync => log::debug!(
                "Created sleeper without a target rate (VSync on)"
            ),
            FrameRateLimit::Unlimited => log::debug!(
                "Created sleeper without a target rate (VSync off)"
            ),
        }
        sleeper
    }

    // Insert the required default systems
    fn insert_default_systems(mut self) -> Self {
        self = self.insert_system(input::system);
        self = self.insert_system(ecs::system);
        self = self.insert_system(world::system);
        self = self.insert_system(utils::threadpool);
        self = self.insert_system(utils::time);
        self = self.insert_system(audio::system);
        self = self.insert_system(networking::system);
        self = self.insert_system(graphics::common);
        self = self.insert_system(graphics::acquire);
        self = self.insert_system(graphics::present);
        self = self.insert_system(rendering::system);

        // Insert the IO manager
        let author = self.author_name.clone();
        let app = self.app_name.clone();
        self = self.insert_system(move |system: &mut System| {
            utils::io(system, author, app)
        });

        // Insert the asset loader
        let user = self.user_assets_folder.take();
        self = self.insert_system(|system: &mut System| {
            assets::system(system, user)
        });

        // Insert the graphics API window resource
        let window_settings = self.window.clone();
        self.world.insert(window_settings);

        // FIXME: Make use of this or remove it
        let app_name = self.app_name.clone();
        let app_version = self.app_version;
        let engine_name = self.engine_name.clone();
        let engine_version = self.engine_version;
        self
    }
}
