//#![windows_subsystem = "windows"]
extern crate gl;
// include the OpenGL type aliases
extern crate glfw;

// World
pub use core;
use std::thread::ThreadId;

// Re-Export
pub use assets;
pub use debug;
pub use defaults;
pub use ecs;
use glfw::Context;
pub use input;
pub use math;
pub use others;
pub use rendering;
pub use terrain;
pub use ui;
pub use veclib;

// Load up the OpenGL window and such
pub fn start(author_name: &str, app_name: &str, assets_preload_callback: fn(), load_entities: fn(), load_systems: fn()) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = glfw
        .create_window(rendering::WINDOW_SIZE.x as u32, rendering::WINDOW_SIZE.y as u32, app_name, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    // Pre-load the assets first
    defaults::preload_default_assets();
    assets_preload_callback();
    // Hehe multithreaded renering goes BRRRRRRRR
    let pipeline_data = rendering::pipec::init_pipeline(&mut glfw, &mut window);
    rendering::pipec::initialize_threadlocal_render_comms();
    // Set the type of events that we want to listen to
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_size_polling(true);
    // Create the world
    core::world::new(author_name, app_name);
    core::world::start_world(&mut glfw, &mut window);
    // Calling the callback
    println!("Calling World Initialization callback");
    defaults::preload_systems();
    load_systems();
    // Get the system thread_ids
    let thread_ids = 
        core::world::world()
        .ecs_manager.systemm.systems
        .iter()
        .map(|x| x
            .join_handle
            .thread()
            .id()
            .clone())
    .collect::<Vec<ThreadId>>();
    others::barrier::init(thread_ids.clone());
    load_entities();
    core::global::main::start_system_loops();
    let mut last_time: f64 = 0.0;
    others::barrier::as_ref().init_finished_world();

    while !window.should_close() {
        // Update the delta_time
        let new_time = glfw.get_time();
        let delta = new_time - last_time;
        last_time = new_time;
        let i = std::time::Instant::now();
        // Update the world
        core::world::update_world_start_barrier(delta);
        // The systems are running, we cannot do anything
        core::world::update_world_end_barrier(delta, &thread_ids);
        // We can do stuff on the main thread
        {
            let mut w = core::world::world_mut();
            let world = &mut *w;
            core::world::update_main_thread_stuff(delta, world, &pipeline_data);
            // Get the GLFW events first
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(key, key_scancode, action_type, _modifiers) => {
                        // Key event
                        let action_id = match action_type {
                            glfw::Action::Press => 0,
                            glfw::Action::Release => 1,
                            glfw::Action::Repeat => 2,
                        };
                        // Only accept the scancode of valid keys
                        if key_scancode > 0 {
                            core::world::receive_key_event(key_scancode, action_id, world);
                        }
                        if let glfw::Key::Escape = key {
                            window.set_should_close(true);
                        }
                    }
                    glfw::WindowEvent::Size(x, y) => {
                        // Size
                        core::world::resize_window_event(x as u16, y as u16, world);
                    }
                    glfw::WindowEvent::Scroll(_, scroll2) => core::world::receive_mouse_scroll_event(scroll2, world),
                    glfw::WindowEvent::CursorPos(x, y) => core::world::receive_mouse_pos_event(x, y, world),
                    _ => {}
                }
            }
            //std::thread::sleep(std::time::Duration::from_millis(16).saturating_sub(i.elapsed()));
        }
    }
    // When the window closes and we exit from the game
    core::world::kill_world(pipeline_data);
    println!("\x1b[31mExiting the engine!\x1b[0m");
}
