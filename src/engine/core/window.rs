extern crate glfw;
use crate::engine::core::world::World;
use crate::gl;
use glfw::{Action, Context, Key};

pub fn setup_window() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	let default_size = World::get_default_window_size();
	let (mut window, events) = glfw
		.create_window(
			default_size.0 as u32,
			default_size.1 as u32,
			"Hypothermia",
			glfw::WindowMode::Windowed,
		)
		.expect("Failed to create GLFW window.");
	gl::load_with(|s| window.get_proc_address(s) as *const _);
	// Set the type of events that we want to listen to
	window.set_key_polling(true);
	window.set_cursor_pos_polling(true);
	window.set_size_polling(true);
	window.make_current();
	if gl::Viewport::is_loaded() {
		println!("OpenGL viewport has loaded");
		unsafe {
			gl::Viewport(0, 0, 300, 300);
		}
	}
	glfw.set_swap_interval(glfw::SwapInterval::None);
	// Create the world
	let mut world: World = World::default();
	world.start_world(&mut window);

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
		glfw::WindowEvent::Key(key, _, action_type, _) => {
			world.input_manager.receive_key_event(key, action_type);
		}
		glfw::WindowEvent::Size(x, y) => {
			world.resize_window_event((x, y));
		}
		_ => {}
	}
}
