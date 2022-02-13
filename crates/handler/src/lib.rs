#![windows_subsystem = "windows"]
use std::sync::Arc;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// include the OpenGL type aliases
extern crate gl;

pub use defaults;
use glutin::{
    dpi::{LogicalSize, LogicalPosition, PhysicalPosition},
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
    ContextBuilder, ContextCurrentState, NotCurrent, WindowedContext,
};
pub use main::*;
use main::{core::{World, WorldTaskReceiver}};
use spin_sleep::LoopHelper;

// Initialize glutin and the window
fn init_glutin_window<U>(el: &EventLoop<U>, title: String) -> WindowedContext<NotCurrent> {
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_visible(true)
        .with_title(title)
        .with_inner_size(LogicalSize::new(
            rendering::utils::DEFAULT_WINDOW_SIZE.x as u32,
            rendering::utils::DEFAULT_WINDOW_SIZE.y as u32,
        ));
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))    
        .build_windowed(wb, el)
        .unwrap();
    let window = wc.window();
    wc
}

// Start le engine
pub fn start(author_name: &str, app_name: &str, preload_assets: fn(), init_world: fn(&mut World)) {
    let event_loop = EventLoop::new();
    let window_context = init_glutin_window(&event_loop, format!("'{}', by '{}'", app_name, author_name));
    // Pre-load the assets first
    defaults::preload_default_assets();
    preload_assets();

    // Load the config file (create it if it doesn't exist already)
    let io = main::io::SaverLoader::new(author_name, app_name);
    io.create_default("config\\game_config.json", &core::GameSettings::default());
    let config: core::GameSettings = io.load("config\\game_config.json");
    io.save("config\\game_config.json", &config);

    // Hehe multithreaded renering goes BRRRRRRRR
    let shadows = config.shadow_resolution.convert();
    let pipeline_settings = rendering::pipeline::PipelineSettings {
        shadow_resolution: shadows.0,
        shadow_bias: shadows.1,
    };
    // A little trolling
    let pipeline_data = rendering::pipeline::init_pipeline(pipeline_settings, window_context);
    // Create the world
    let mut task_receiver = core::WorldTaskReceiver::new();
    let mut world = World::new(config, io, pipeline_data);

    // Init the world
    // Calling the callback
    println!("Calling World Initialization callback");
    {
        // Load the default systems first
        defaults::preload_system(&mut world);
        init_world(&mut world);
        // Flush everything and execute all the tasks
        task_receiver.flush(&mut world);
    }
    println!("Hello Game World!");
    let mut sleeper = LoopHelper::builder().build_with_target_rate(300.0);

    event_loop.run(move |event, _, control_flow| {
        // Winit
        *control_flow = ControlFlow::Poll;
        // Handle the glutin (winit) events
        handle_glutin_events(&mut sleeper, &mut task_receiver, &mut world, event, control_flow);
    });
}
// Handle events
fn handle_glutin_events(sleeper: &mut LoopHelper, task_receiver: &mut WorldTaskReceiver, world: &mut World, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
        // Window events
        Event::WindowEvent { window_id: _, event } => handle_window_event(event, world, control_flow),
        // Loop events
        Event::MainEventsCleared => {
            // Update the delta time
            let delta = sleeper.loop_start_s();
            // Update the timings
            world.time.update(delta);
            // We can update the world now
            world.update_start(task_receiver);
            world.update_end(task_receiver);
        }
        // When we exit from the engine
        Event::LoopDestroyed => {
            // When the window closes and we exit from the game
            println!("Exiting the engine...");
            world.destroy();
            println!("\x1b[31mThe sense of impending doom is upon us.\x1b[0m");
        }

        _ => (),
    }
}

// Handle the window events
fn handle_window_event(event: WindowEvent, world: &mut World, control_flow: &mut ControlFlow) {
    match event {
        glutin::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        glutin::event::WindowEvent::Resized(size) => world.resize_window_event(veclib::vec2(size.width as u16, size.height as u16)),
        glutin::event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
            if let Some(virtual_keycode) = input.virtual_keycode {
                world.input.receive_key_event(virtual_keycode, input.state);
                // Exit when we press the escape key
                if let glutin::event::VirtualKeyCode::Escape = virtual_keycode {
                    *control_flow = ControlFlow::Exit;
                }
            }
        }
        glutin::event::WindowEvent::CursorMoved {
            device_id: _,
            position,
            modifiers: _,
        } => { world.input.receive_mouse_position_event(veclib::vec2(position.x, position.y)); },
        glutin::event::WindowEvent::MouseWheel {
            device_id: _,
            delta,
            phase: _,
            modifiers: _,
        } => {
            match delta {
                glutin::event::MouseScrollDelta::LineDelta(x, y) => world.input.receive_mouse_scroll_event(y as f64),
                glutin::event::MouseScrollDelta::PixelDelta(y) => world.input.receive_mouse_scroll_event(y.x),
            }
        }
        _ => {}
    }
}