//#![windows_subsystem = "windows"]
extern crate gl;
// include the OpenGL type aliases
extern crate glfw;

// World
pub use core::World;

// Re-Export
pub use debug;
pub use defaults;
pub use ecs;
pub use errors;
use glfw::Context;
pub use input;
pub use math;
pub use others;
pub use rendering;
pub use resources;
pub use world_data;
pub use systems;
pub use terrain;
pub use ui;
pub use veclib;

// Load up the OpenGL window and such
pub fn start(author_name: &str, app_name: &str, callback: fn(&mut World)) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let default_size = others::get_default_window_size();
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
    if gl::Viewport::is_loaded() {
        unsafe {
            gl::Viewport(0, 0, 300, 300);
        }
    }
    // Create the world
    let mut world: World = World::new(author_name, app_name);
    world.start_world(&mut glfw, &mut window, callback);
    let mut last_time: f64 = 0.0;

    while !window.should_close() {
        // Update the delta_time
        let new_time = glfw.get_time();
        let delta = new_time - last_time;
        last_time = new_time;
        // Update the world
        world.update_world(&mut window, &mut glfw, delta);

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
        glfw::WindowEvent::Scroll(_scroll, scroll2) => world.input_manager.receive_mouse_event(None, Some(scroll2)),
        glfw::WindowEvent::CursorPos(x, y) => world.input_manager.receive_mouse_event(Some((x, y)), None),
        _ => {}
    }
}
