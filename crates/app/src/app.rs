use crate::handler;
use glutin::event_loop::EventLoop;
use rendering::context::Graphics;
use std::path::PathBuf;
use world::{Descriptor, Events, Init, System, Update, World};

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    screensize: vek::Extent2<u16>,
    fullscreen: bool,
    vsync: bool,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,

    // These are the events that will contained within the world
    events: Events,

    // This is the main world that is instantiated once
    world: World,
}

impl Default for App {
    fn default() -> Self {
        // Only called once
        let (world, events) = world::setup();

        Self {
            title: "Default title".to_string(),
            screensize: vek::Extent2::new(1280, 720),
            fullscreen: false,
            vsync: false,
            user_assets_folder: None,
            events,
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

    // Set the window vsync toggle
    pub fn set_window_vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
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
    pub fn insert_system(mut self, system: impl System) -> Self {
        system.insert(&mut self.events);
        self
    }

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Prepare the event loop
        let el = EventLoop::new();

        // Insert all the builtin systems
        self = self.insert_system(input::system);
        self = self.insert_system(gui::system);
        self = self.insert_system(ecs::system);
        self = self.insert_system(world::system);
        self = self.insert_system(|world| assets::system(world, user));        
        //assets::system(world.events(), user)

        // Run le game engine
        //handler::run(el, self.updates, world);
    }
}
