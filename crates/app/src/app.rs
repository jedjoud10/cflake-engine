use world::World;

// An app is just a world builder. It uses the builder pattern to construct a world object and the corresponding game engine window
pub struct App {
    // Window settings for the graphics
    title: String,
    fullscreen: bool,
    screensize: vek::Extent2<u16>,
    vsync: bool,

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
    pub fn sort(mut self) -> Self {
        // One sorting function that will be used twice
        fn sort(vec: &mut Vec<(fn(&mut World), i32)>) {
           vec.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
        } 

        // Don't care + L + ratio        
        sort(&mut self.startup_systems);
        sort(&mut self.update_systems);
        self
    }

    // Start the engine and consume the app
    pub fn execute(mut self) {
        // Prepare for execution
        self = self.sort();
        
        // Create the world and ececute the event loop
        let world = World::default();
    }
}

/*
// Start le engine
pub fn start(title: impl Into<String>, init_world: fn(&mut World)) {
    // Load the config file (create it if it doesn't exist already)
    let title: String = title.into();
    let io = io::IOManager::new(&title);
    let config: Settings = io.load("config/engine.json").unwrap_or_else(|_| {
        // If we failed reading the config file, try creating it and saving it
        io.create_file("config/engine.json");
        io.save("config/engine.json", &Settings::default());
        Settings::default()
    });

    // Glutin stuff
    let event_loop = EventLoop::new();

    // Preload the default assets
    defaults::preload_default_assets();

    // Since the pipeline also handles OpenGL context, we should make the window context using the pipeline
    let shadow = config.shadows;
    let ws = config.window.clone();

    // TODO: Shit ugly: fix
    let (pipeline, renderer) = rendering::pipeline::new(
        &event_loop,
        title,
        ws.fps_cap == FrameRateCap::Vsync,
        ws.fullscreen,
        PipelineSettings::new((shadow.resolution() != 0).then(|| shadow)),
    );

    // Create the world
    let mut world = World::new(config, io, pipeline, renderer);

    // Load the default systems first
    defaults::load_default_systems(&mut world);
    EventExecutionOrder::set(0);
    init_world(&mut world);
    world.events.sort();

    // Post-init
    world.pipeline.post_init();

    // FPS cap
    let builder = LoopHelper::builder();
    let mut sleeper = match ws.fps_cap {
        FrameRateCap::Unlimited => builder.build_without_target_rate(),
        FrameRateCap::Limited(cap) => {
            assert!(cap != 0, "Frame rate limit cannot be zero");
            builder.build_with_target_rate(cap as f32)
        }
        FrameRateCap::Vsync => builder.build_without_target_rate(),
    };

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        // Handle the glutin (winit) events
        handle_glutin_events(&mut sleeper, &mut world, event, control_flow);
    });
}


// Handle events
fn handle_glutin_events(sleeper: &mut LoopHelper, world: &mut World, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
        // Window events
        Event::WindowEvent { window_id: _, event } => {
            if !world.input.is_accepting_input() {
                world.gui.receive_event(&event);
            }
            world.pipeline.handle_window_event(&mut world.renderer, &event, control_flow);
        }
        // Device event
        Event::DeviceEvent { device_id: _, event } => handle_device_event(event, world, control_flow),
        // Loop events
        Event::MainEventsCleared => {
            // Update the delta time
            let delta = sleeper.loop_start_s() as f32;
            // We can update the world now
            world.update(delta);

            // If the world state is "exit", we must exit from the game
            if let WorldState::Exit = world.state {
                *control_flow = ControlFlow::Exit
            }
            sleeper.loop_sleep();
        }
        // When we exit from the engine
        Event::LoopDestroyed => {
            // When the window closes and we exit from the game
            println!("Exiting the engine...");
            println!("The sense of impending doom is upon us.");
            world.destroy();
        }

        _ => (),
    }
}

// Handle device events
fn handle_device_event(event: DeviceEvent, world: &mut World, _control_flow: &mut ControlFlow) {
    match event {
        DeviceEvent::MouseMotion { delta } => {
            if world.input.is_accepting_input() {
                world.input.receive_mouse_position_event(vek::Vec2::new(delta.0 as f32, delta.1 as f32));
            }
        }
        DeviceEvent::MouseWheel { delta } => {
            if world.input.is_accepting_input() {
                match delta {
                    glutin::event::MouseScrollDelta::LineDelta(_x, y) => world.input.receive_mouse_scroll_event(y),
                    glutin::event::MouseScrollDelta::PixelDelta(y) => world.input.receive_mouse_scroll_event(y.x as f32),
                }
            }
        }
        DeviceEvent::Key(input) => {
            if let Some(virtual_keycode) = input.virtual_keycode {
                world.input.receive_key_event(virtual_keycode, input.state);
            }
        }
        _ => (),
    }
}
*/
