use std::path::PathBuf;

use glutin::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, DeviceEvent}};
use rendering::context::Graphics;
use world::World;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    fullscreen: bool,
    screensize: vek::Extent2<u16>,
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
            fullscreen: false,
            screensize: vek::Extent2::new(1920, 1080),
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

    // Sort all the systems into their respective orders (startups and updates)
    fn sort(&mut self) {
        // One sorting function that will be used twice
        fn sort(vec: &mut Vec<(fn(&mut World), i32)>) {
            vec.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
        }

        // Don't care + L + ratio
        sort(&mut self.startup_systems);
        sort(&mut self.update_systems);
    }

    // Consume the App builder, and start the engine window
    pub fn execute(mut self) {
        // Prepare the world and the even loop
        self.sort();
        let el = EventLoop::new();
        let mut world = World::default();

        // Create the graphics pipeline
        let (el, graphics) = Graphics::new(el);

        // Insert the default main resources
        world.insert(graphics);
        world.insert(ecs::EcsManager::default());
        world.insert(input::Input::default());
        world.insert(assets::Assets::new(self.user_assets_folder));

        // Run le game engine
        run(el, world);        
    }
}

// Run the event loop, and start displaying the game engine window
fn run(el: EventLoop<()>, mut world: World) {
    el.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id: _, event } => window(&mut world, event),
            Event::DeviceEvent { device_id: _, event } => device(&mut world, event),
            Event::MainEventsCleared => update(&mut world),
            _ => (),
        }
    })
}

// Handle new window events
fn window(world: &mut World, event: WindowEvent) {

}

// Handle new device events
fn device(world: &mut World, device: DeviceEvent) {

}


// Execute one step-frame of the engine
fn update(world: &mut World) {

}