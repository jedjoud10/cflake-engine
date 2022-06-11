use std::path::PathBuf;

use glutin::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, DeviceEvent}};
use rendering::context::Graphics;
use world::World;

use crate::handler;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    screensize: vek::Extent2<u16>,
    fullscreen: bool,
    vsync: bool,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,

    // Systems that will be executed at the start/each frame
    startup_systems: Vec<(fn(&mut World), i32)>,
    update_systems: Vec<(fn(&mut World), i32)>,

    // System ordering
    startup_idx: i32,
    update_idx: i32,
}

impl App {
    // Create a new world builder
    pub fn new() -> Self {
        Self {
            title: "Default title".to_string(),
            screensize: vek::Extent2::new(1280, 720),
            fullscreen: false,
            vsync: false,
            user_assets_folder: None,
            startup_systems: Vec::new(),
            update_systems: Vec::new(),
            startup_idx: 0,
            update_idx: 0,
        }
    }

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

    // Insert a startup system into the application that will execute once we begin
    pub fn insert_startup(mut self, system: fn(&mut World)) -> Self {
        self.startup_idx += 1;
        let copy = self.startup_idx;
        self.insert_startup_with(system, copy)
    }

    // Insert a startup system with a specific ordering index
    pub fn insert_startup_with(mut self, system: fn(&mut World), order: i32) -> Self {
        self.startup_systems.push((system, order));
        self
    }

    // Insert an update system that will execute each frame
    pub fn insert_update(mut self, system: fn(&mut World)) -> Self {
        self.update_idx += 1;
        let copy = self.update_idx;
        self.insert_update_with(system, copy)
    }

    // Insert an update system with a specific ordering index
    pub fn insert_update_with(mut self, system: fn(&mut World), order: i32) -> Self {
        self.update_systems.push((system, order));
        self
    }

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Prepare the world and the even loop
        let el = EventLoop::new();
        let mut world = World::default();

        // Create the graphics pipeline
        let (el, graphics) = Graphics::new(el, self.title, self.screensize, self.fullscreen, self.vsync);

        // Insert the default main resources
        world.insert(graphics);
        world.insert(ecs::EcsManager::default());
        world.insert(input::Input::default());
        world.insert(assets::Assets::new(self.user_assets_folder));

        // One sorting function that will be used twice
        fn sort(vec: &mut Vec<(fn(&mut World), i32)>) -> Vec<fn(&mut World)> {
            vec.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
            vec.into_iter().map(|(system, _)| system.clone()).collect::<Vec<_>>()
        }

        // Don't care + L + ratio
        let startups = sort(&mut self.startup_systems);
        let updates = sort(&mut self.update_systems);

        // Run the init systems before starting the engine window
        for system in startups {
            system(&mut world);
        }

        // Run le game engine
        handler::run(el, updates, world);        
    }
}

