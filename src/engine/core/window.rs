extern crate glfw;
use crate::engine::core::world::World;
use crate::gl;
use glfw::{Action, Context, Key};

pub fn setup_window() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	let default_size = World::get_default_window_size();
    let (mut window, events) = glfw.create_window(default_size.0, default_size.1, "Hypothermia", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

	gl::load_with(|s| window.get_proc_address(s) as *const _);
    window.set_key_polling(true);
    window.make_current();

	if gl::Viewport::is_loaded() {
		println!("OpenGL viewport has loaded");
		unsafe {
			gl::Viewport(0, 0, 300, 300);
		}
	}

	// Create the world
	let mut world: World = World::default();
	world.start_world();

    while !window.should_close() {

		// Read the events at the start of the frame
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, &mut world, event);
        }	

		// Update the delta_time
		let new_time = glfw.get_time();
		world.time_manager.delta_time = new_time - world.time_manager.time_since_start;
		world.time_manager.time_since_start = new_time;
		// Update the world
		world.update_world(&mut window, &mut glfw);


		window.swap_buffers();
    }	
	// When the window closes and we exit from the game
	world.stop_world();
}

fn handle_window_event(window: &mut glfw::Window, world: &mut World, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(key, _, action_type, _) => {
            world.input_manager.receive_key_event(key, action_type);
        }
        _ => {}
    }
}