use std::path::PathBuf;
use glutin::event_loop::EventLoop;
use rendering::context::Graphics;
use world::{World, Events, System, Init, Update, WindowEvent, DeviceEvent, Descriptor, BoxedEvent};
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

    // These are the events that will contained within the world
    events: Events,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Default title".to_string(),
            screensize: vek::Extent2::new(1280, 720),
            fullscreen: false,
            vsync: false,
            user_assets_folder: None,
            events: Default::default()
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

    // Register a new event using it's marker descriptor and it's priority index
    pub fn register_with<'a, Marker: Descriptor<'a>>(
        mut self,
        event: impl BoxedEvent<Marker> + 'static,
        priority: i32,
    ) -> Self {
        self.events.register_with(event, priority);
        self
    }
    
    // Register a new event using it's marker descriptor and an automatic priority index
    pub fn register<'a, Marker: Descriptor<'a>>(mut self, event: impl BoxedEvent<Marker> + 'static) -> Self {
        self.events.register(event);
        self
    }

    // Insert a new system into the application and execute it immediately
    // Systems are subsests of events that allow to organize them more specifically, like RenderSystem or InputSystem (even though they are just closures)
    pub fn insert_system(mut self, system: impl System) -> Self {
        system.insert(&mut self.events);
        self
    }

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Prepare the event loop and create the main world
        let el = EventLoop::new();
        let mut world = World::default();
        *world.events() = self.events;

        // Insert all the builtin systems
        self.insert_system(input::system);
        //assets::system(world.events(), user)

        // Sort all the builtint events
        let events = world.events();
        events.sort::<Init>();
        events.sort::<Update>();
        events.sort::<WindowEvent>();
        events.sort::<DeviceEvent>();

        // Execute the init events
        events.execute::<Init>(world);

        // Run le game engine
        //handler::run(el, self.updates, world);
    }
}
