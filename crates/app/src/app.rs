use ahash::AHashSet;
use glutin::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
//use gui::egui::util::id_type_map::TypeId;
use mimalloc::MiMalloc;
//use rendering::prelude::{FrameRateLimit, GraphicsSetupSettings};
use std::{path::PathBuf, any::TypeId};
use world::{Event, Events, Init, State, System, Update, World};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    screensize: vek::Extent2<u16>,
    fullscreen: bool,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,

    // Main app resources
    events: Events,
    systems: AHashSet<TypeId>,
    world: World,
    el: EventLoop<()>,
}

impl Default for App {
    fn default() -> Self {
        let (world, events) = world::setup();

        Self {
            title: "Default title".to_string(),
            screensize: vek::Extent2::new(1280, 720),
            fullscreen: false,
            //limit: None,
            user_assets_folder: None,
            events,
            systems: Default::default(),
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

    // Set the window starting screensize
    pub fn set_window_size(mut self, size: vek::Extent2<u16>) -> Self {
        self.screensize = size;
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
    pub fn insert_system<S: System>(mut self, system: S) -> Self {
        if self.systems.insert(TypeId::of::<S>()) {
            system.insert(&mut self.events);
        }
        self
    }

    // Insert a single update event
    pub fn insert_update<P>(mut self, update: impl Event<Update, P>) -> Self {
        self.events.registry::<Update>().insert(update);
        self
    }

    // Insert a single init event
    pub fn insert_init<P>(mut self, update: impl Event<Init, P>) -> Self {
        self.events.registry::<Init>().insert(update);
        self
    }

    // Insert a single window event
    pub fn insert_window<P>(mut self, update: impl Event<WindowEvent<'static>, P>) -> Self {
        self.events.registry::<WindowEvent>().insert(update);
        self
    }

    // Insert a single device event
    pub fn insert_device<P>(mut self, update: impl Event<DeviceEvent, P>) -> Self {
        self.events.registry::<DeviceEvent>().insert(update);
        self
    }

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
        let reg = self.events.registry::<Init>();
        reg.sort().unwrap();
        self.events.registry::<Init>().execute((&mut self.world, &self.el));

        // Decompose the app
        let mut events = self.events;
        let mut world = self.world;
        let el = self.el;

        // Sort the remaining events registries
        events.registry::<Update>().sort().unwrap();
        events.registry::<WindowEvent>().sort().unwrap();
        events.registry::<DeviceEvent>().sort().unwrap();

        // We must now start the game engine (start the glutin event loop)
        el.run(move |event, _, cf| match event {
            // Call the update events
            glutin::event::Event::MainEventsCleared => {
                events.registry::<Update>().execute(&mut world);

                if let State::Stopped = *world.get::<State>().unwrap() {
                    *cf = ControlFlow::Exit;
                }

            }

            // Call the window events
            glutin::event::Event::WindowEvent {
                window_id: _,
                mut event,
            } => {
                events.registry::<WindowEvent>().execute((&mut world, &mut event));
            }

            // Call the device events
            glutin::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                events.registry::<DeviceEvent>().execute((&mut world, &event));
            }
            _ => {}
        });
    }
}
