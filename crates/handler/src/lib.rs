#![windows_subsystem = "windows"]
use std::sync::Arc;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// include the OpenGL type aliases
extern crate gl;

pub use defaults;
use main::{core::World, rendering::utils::WindowInitSettings};
pub use main::*;
use spin_sleep::LoopHelper;
use winit::{event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow}, dpi::LogicalSize, window::{WindowBuilder, Window}, error::OsError, event::Event};

// Initialize winit and the window
fn init_winit_window<T>(window_target: &EventLoopWindowTarget<T>, title: String) -> Result<Window, OsError> {
    WindowBuilder::new()
        .with_resizable(true)
        .with_visible(true)
        .with_title(title)
        .with_inner_size(LogicalSize::new(rendering::utils::DEFAULT_WINDOW_SIZE.x as u32, rendering::utils::DEFAULT_WINDOW_SIZE.y as u32))
        .build(window_target)
}

// Load up the OpenGL window and such
pub fn start(author_name: &str, app_name: &str, preload_assets: fn(), init_world: fn(&mut World)) {
    let event_loop = EventLoop::new();
    let window = init_winit_window(&event_loop, format!("'{}', by '{}'", app_name, author_name)).unwrap();
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
    let pipelin_settings = rendering::pipeline::PipelineSettings {
        shadow_resolution: shadows.0,
        shadow_bias: shadows.1,
    };
    let window_init_settings = WindowInitSettings {
        dimensions: {
            let fs = window.inner_size();
            veclib::vec2(fs.width as u16, fs.height as u16)
        },
        pixel_per_point: window.scale_factor() as f32,
    };
    // A little trolling
    let window = Arc::new(window);
    let pipeline_data = rendering::pipeline::init_pipeline(pipelin_settings, window.clone());
    // Create the world
    let mut task_receiver = core::WorldTaskReceiver::new();
    let mut world = World::new(window_init_settings, config, io, pipeline_data);

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
    let mut sleeper = LoopHelper::builder().build_with_target_rate(240.0);

    event_loop.run(move |event, _, control_flow| {
        // Winit
        *control_flow = ControlFlow::Poll;

        // Update the delta time
        let delta = sleeper.loop_start_s();
        // Update the timings
        world.time.update(delta);
        // Handle the winit events
        handle_winit_event(&mut world, event, control_flow, window.as_ref());

        // We can update the world now
        world.update_start(&mut task_receiver);
        world.update_end(&mut task_receiver);
        //sleeper.();
    });
    // When the window closes and we exit from the game
    println!("Exiting the engine...");
    world.destroy();
    println!("\x1b[31mThe sense of impending doom is upon us.\x1b[0m");
}
// Handle the winit events
fn handle_winit_event(world: &mut World, event: Event<()>, control_flow: &mut ControlFlow, window: &Window) {
    match event {
        // Event loop events
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        Event::RedrawRequested(_) => {

        },

        // Window events
        Event::WindowEvent { window_id, event } => {
            match event {
                winit::event::WindowEvent::Resized(size) => world.resize_window_event(veclib::vec2(size.width as u16, size.height as u16)),
                //winit::event::WindowEvent::CloseRequested => window.cl,
                //init::event::WindowEvent::Destroyed => todo!(),
                //winit::event::WindowEvent::Focused(_) => todo!(),
                winit::event::WindowEvent::KeyboardInput { device_id, input, is_synthetic } => { world.input.receive_key_event(input.virtual_keycode.unwrap(), input.state); },
                winit::event::WindowEvent::CursorMoved { device_id, position, modifiers: _ } => world.input.receive_mouse_position_event(veclib::vec2(position.x, position.y)),
                winit::event::WindowEvent::MouseWheel { device_id, delta, phase, modifiers: _ } => /* world.input.receive_mouse_scroll_event(delta.) */ {},
                //winit::event::WindowEvent::MouseInput { device_id, state, button, modifiers: _ } => todo!(),
                _ => {}
            }
        }
        _ => ()
    }
    /*
    for (_, event) in glfw::flush_messages(events) {
        match event.clone() {
            glfw::WindowEvent::Key(key, key_scancode, action_type, _modifiers) => {
                // Key event
                let action_id = match action_type {
                    glfw::Action::Press => 0,
                    glfw::Action::Release => 1,
                    glfw::Action::Repeat => 2,
                };
                // Only accept the scancode of valid keys
                if key_scancode > 0 {
                    world.input.receive_key_event(key_scancode, action_id);
                }
                if let glfw::Key::Escape = key {
                    window.set_should_close(true);
                }
            }
            glfw::WindowEvent::Size(x, y) => world.resize_window_event(veclib::Vector2::new(x as u16, y as u16)),
            glfw::WindowEvent::Scroll(_, scroll) => world.input.receive_mouse_scroll_event(scroll),
            glfw::WindowEvent::CursorPos(x, y) => world.input.receive_mouse_position_event((x, y)),
            glfw::WindowEvent::Close => window.set_should_close(true),
            _ => {}
        }
    }
    */
}
