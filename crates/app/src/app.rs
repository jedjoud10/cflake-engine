use ahash::AHashSet;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
//use gui::egui::util::id_type_map::TypeId;
use mimalloc::MiMalloc;
use rendering::prelude::FrameRateLimit;
use std::{any::TypeId, path::PathBuf};
use world::{Event, Systems, Init, Shutdown, State, System, Update, World};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    limit: FrameRateLimit,
    fullscreen: bool,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,

    // Main app resources
    systems: Systems,
    world: World,
    el: EventLoop<()>,
}

impl Default for App {
    fn default() -> Self {
        let (world, systems) = world::setup();

        Self {
            title: "Default title".to_string(),
            limit: FrameRateLimit::Umlimited,
            fullscreen: false,
            user_assets_folder: None,
            systems,
            el: EventLoop::new(),
            world,
        }
    }
}

impl App {
    // Set the window title
    pub fn set_window_title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    // Set the window framerate limit
    pub fn set_frame_rate_limit(mut self, limit: FrameRateLimit) -> Self {
        self.limit = limit;
        self
    }

    // Set window fullscreen mode
    pub fn set_window_fullscreen(mut self, toggled: bool) -> Self {
        self.fullscreen = toggled;
        self
    }

    // Set the assets folder for the user defined assets
    pub fn set_user_assets_folder_path(mut self, path: impl TryInto<PathBuf>) -> Self {
        self.user_assets_folder = Some(
            path.try_into()
                .ok()
                .expect("Input path failed to convert into PathBuf"),
        );
        self
    }

    // Insert a new system into the app and execute it immediately
    // This will register all the necessary events automatically
    pub fn insert_system(mut self, callback: impl FnOnce(&mut System)) -> Self {
        self.systems.insert(callback);
        self
    }

    /*



    // Insert a single update event
    pub fn insert_update<ID>(mut self, update: impl Event<Update, ID>) -> Self {
        self.events.registry_mut::<Update>().insert(update);
        self
    }

    // Insert a single init event
    pub fn insert_init<ID>(mut self, update: impl Event<Init, ID>) -> Self {
        self.events.registry_mut::<Init>().insert(update);
        self
    }

    // Insert a single window event
    pub fn insert_window<ID>(mut self, update: impl Event<WindowEvent<'static>, ID>) -> Self {
        self.events.registry_mut::<WindowEvent>().insert(update);
        self
    }

    // Insert a single device event
    pub fn insert_device<ID>(mut self, update: impl Event<DeviceEvent, ID>) -> Self {
        self.events.registry_mut::<DeviceEvent>().insert(update);
        self
    }

    // Insert a single exit event
    pub fn insert_exit<ID>(mut self, exit: impl Event<Shutdown, ID>) -> Self {
        self.events.registry_mut::<Shutdown>().insert(exit);
        self
    }
    */

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Insert all the builtin systems dataless
        self = self
            .insert_system(input::system)
            .insert_system(ecs::system)
            .insert_system(time::system)
            .insert_system(world::system);

        // Insert the asset loader
        let user = self.user_assets_folder.take();
        self = self.insert_system(|e: &mut Events| assets::system(e, user));

        // Sort & execute the init events
        let reg = self.events.registry_mut::<Init>();
        reg.sort().unwrap();
        self.events
            .registry_mut::<Init>()
            .execute((&mut self.world, &self.el));

        // Decompose the app
        let mut events = self.events;
        let mut world = self.world;
        let el = self.el;

        // Sort the remaining events registries
        events.registry_mut::<Update>().sort().unwrap();
        events.registry_mut::<WindowEvent>().sort().unwrap();
        events.registry_mut::<DeviceEvent>().sort().unwrap();

        // Create the spin sleeper for frame limiting
        let builder = spin_sleep::LoopHelper::builder();
        let mut sleeper = if let FrameRateLimit::Limited(limit) = self.limit {
            builder.build_with_target_rate(limit)
        } else {
            builder.build_without_target_rate()
        };

        // We must now start the game engine (start the winit event loop)
        el.run(move |event, _, cf| match event {
            // Call the update events
            winit::event::Event::MainEventsCleared => {
                sleeper.loop_start();
                events.registry_mut::<Update>().execute(&mut world);
                if let State::Stopped = *world.get::<State>().unwrap() {
                    *cf = ControlFlow::Exit;
                }
                sleeper.loop_sleep();
            }

            // Call the window events
            winit::event::Event::WindowEvent {
                window_id: _,
                mut event,
            } => {
                events
                    .registry_mut::<WindowEvent>()
                    .execute((&mut world, &mut event));
            }

            // Call the device events
            winit::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                events
                    .registry_mut::<DeviceEvent>()
                    .execute((&mut world, &event));
            }
            _ => {}
        });
    }
}
