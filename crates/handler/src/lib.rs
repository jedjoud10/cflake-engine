use defaults::{rendering::pipeline::PipelineSettings, ecs::system::SystemExecutionOrder};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use defaults;
use glutin::{
    event::{DeviceEvent, Event},
    event_loop::{ControlFlow, EventLoop},
};
use spin_sleep::LoopHelper;
pub use world::*;

// Start le engine
pub fn start(author_name: &str, app_name: &str, init_world: fn(&mut World), init_systems: fn(&mut World)) {
    // Load the config file (create it if it doesn't exist already)
    let io = io::IOManager::new(author_name, app_name);
    let config: Settings = io.load("config/game_config.json").unwrap_or_else(|_| {
        // If we failed reading the config file, try creating it and saving it
        io.create_file("config/game_config.json");
        io.save("config/game_config.json", &Settings::default());
        Settings::default()
    });

    // Glutin stuff
    let event_loop = EventLoop::new();

    // Preload the default assets
    defaults::preload_default_assets();

    // Since the pipeline also handles OpenGL context, we should make the window context using the pipeline
    let shadows = config.shadow_resolution.convert();
    let (pipeline, renderer) = rendering::pipeline::new(
        &event_loop,
        format!("'{}', by '{}'", app_name, author_name),
        config.vsync,
        config.fullscreen,
        PipelineSettings {
            shadow_resolution: if shadows.0 == 0 { None } else { Some(shadows.0) },
            shadow_bias: shadows.1,
        },
    );

    // Create the world
    let mut world = World::new(config, io, pipeline, renderer);

    // Load the default systems first
    defaults::load_default_systems(&mut world);
    SystemExecutionOrder::set(0);
    init_systems(&mut world);
    println!("Calling World Initialization callback");
    init_world(&mut world);
    world.ecs.systems.sort();

    // Post-init
    world.pipeline.post_init();

    let mut sleeper = if config.fps_cap <= 0 {
        LoopHelper::builder().build_without_target_rate()
    } else {
        LoopHelper::builder().build_with_target_rate(config.fps_cap as f32)
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
            world.gui.receive_event(&event);
            world.pipeline.handle_window_event(&mut world.renderer, event, control_flow);
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
            world.input.receive_mouse_position_event(vek::Vec2::new(delta.0 as f32, delta.1 as f32));
        }
        DeviceEvent::MouseWheel { delta } => match delta {
            glutin::event::MouseScrollDelta::LineDelta(_x, y) => world.input.receive_mouse_scroll_event(y),
            glutin::event::MouseScrollDelta::PixelDelta(y) => world.input.receive_mouse_scroll_event(y.x as f32),
        },
        DeviceEvent::Key(input) => {
            if let Some(virtual_keycode) = input.virtual_keycode {
                world.input.receive_key_event(virtual_keycode, input.state);
            }
        }
        _ => (),
    }
}
