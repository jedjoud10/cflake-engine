use mimalloc::MiMalloc;
use rendering::pipeline::*;
use settings::*;
pub use world::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use defaults;
use glutin::{
    event::{DeviceEvent, Event},
    event_loop::{ControlFlow, EventLoop},
};
use spin_sleep::LoopHelper;
use world::ecs::EventExecutionOrder;

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
    let shadows = config.shadows.resolution.convert();
    let ws = config.window.clone();
    let (pipeline, renderer) = rendering::pipeline::new(
        &event_loop,
        title,
        ws.fps_cap == FrameRateCap::Vsync,
        ws.fullscreen,
        PipelineSettings {
            shadow_resolution: if shadows.0 == 0 { None } else { Some(shadows.0) },
            shadow_bias: shadows.1,
        },
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
        Event::DeviceEvent { device_id: _, event } => {
            // Only handle device events if the window is in focus
            if world.pipeline.window().focused() {
                handle_device_event(event, world, control_flow)
            }
        }
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
