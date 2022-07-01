use glutin::{event_loop::{EventLoop, ControlFlow}, event::{DeviceEvent, WindowEvent}};
use rendering::prelude::GraphicsSetupSettings;
use std::path::PathBuf;
use world::{Events, System, World, Init, Update};

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    screensize: vek::Extent2<u16>,
    fullscreen: bool,
    vsync: bool,

    // Asset and IO
    user_assets_folder: Option<PathBuf>,

    // Main app resources
    events: Events,
    world: World,
    el: EventLoop<()>,
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
        // Insert all the builtin systems dataless
        self = self
            .insert_system(input::system)
            .insert_system(gui::system)
            .insert_system(ecs::system)
            .insert_system(time::system)
            .insert_system(world::system);

        // Insert the asset loader
        let user = self.user_assets_folder.take();
        self = self.insert_system(|e: &mut Events| assets::system(e, user));

        // Insert the graphics pipeline and everything rendering related
        let settings = GraphicsSetupSettings {
            title: self.title.clone(),
            size: self.screensize,
            fullscreen: self.fullscreen,
            vsync: self.vsync,
        };
        self = self.insert_system(|e: &mut Events| rendering::scene::system(e, settings));

        // Sort & execute the init events
        let mut reg = self.events.registry::<Init>();
        reg.sort().unwrap();
        reg.execute((&mut self.world, &mut self.el));

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
            glutin::event::Event::MainEventsCleared => {
                // Call the update events
                events.registry::<Update>().execute(&mut world);

                // Update the current control flow based on the world state
            },
            glutin::event::Event::WindowEvent { window_id, mut event } => {
                // Call the window events
                events.registry::<WindowEvent>().execute((&mut world, &mut event));
            },
            glutin::event::Event::DeviceEvent { device_id, event } => {
                // Call the device events
                events.registry::<DeviceEvent>().execute((&mut world, &event));
            },
            _ => {}
        });
    }
}