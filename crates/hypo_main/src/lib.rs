//#![windows_subsystem = "windows"]
extern crate gl;
// include the OpenGL type aliases
extern crate glfw;

// World
pub use hypo_core::World;

// Export
use glfw::Context;
pub use hypo_debug;
pub use hypo_defaults::components;
pub use hypo_defaults::systems;
pub use hypo_ecs::*;
pub use hypo_errors::*;
pub use hypo_input::*;
pub use hypo_others::*;
pub use hypo_rendering::*;
pub use hypo_resources::*;
pub use hypo_system_event_data::*;
pub use hypo_systems::*;
pub use hypo_terrain::*;
pub use veclib;

pub fn start(load_systems_callback: fn(&mut World), load_entities_callback: fn(&mut World)) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let default_size = hypo_others::get_default_window_size();
    let (mut window, events) = glfw
        .create_window(default_size.0 as u32, default_size.1 as u32, "Hypothermia", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    // Set the type of events that we want to listen to
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_size_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    if gl::Viewport::is_loaded() {
        unsafe {
            gl::Viewport(0, 0, 300, 300);
        }
    }
    // Create the world
    let mut world: World = World::default();
    world.start_world(&mut window, load_systems_callback, load_entities_callback);

    while !window.should_close() {
        // Update the delta_time
        let new_time = glfw.get_time();
        world.time_manager.delta_time = new_time - world.time_manager.seconds_since_game_start;
        world.time_manager.seconds_since_game_start = new_time;
        // Update the world
        world.update_world(&mut window, &mut glfw);

        // Read the events at the start of the frame
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, &mut world, event);
        }
    }
    // When the window closes and we exit from the game
    world.kill_world();
}

// When the window receives a new event
fn handle_window_event(_window: &mut glfw::Window, world: &mut World, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(_, key_scancode, action_type, modifiers) => {
            // Key event
            let action_id = match action_type {
                glfw::Action::Press => 0,
                glfw::Action::Release => 1,
                glfw::Action::Repeat => 2,
            };
            // Only accept the scancode of valid keys
            if key_scancode > 0 {
                world.input_manager.receive_key_event(key_scancode, action_id);
            }
        }
        glfw::WindowEvent::Size(x, y) => {
            // Size
            world.resize_window_event((x as u16, y as u16));
        }
        glfw::WindowEvent::Scroll(_scroll, scroll2) => world.input_manager.recieve_mouse_event(None, Some(scroll2)),
        glfw::WindowEvent::CursorPos(x, y) => world.input_manager.recieve_mouse_event(Some((x, y)), None),
        _ => {}
    }
}
