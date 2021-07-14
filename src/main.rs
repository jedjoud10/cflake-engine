extern crate glfw;
mod world;
use glfw::{Action, Context, Key};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(300, 300, "Hypothermia", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

	// Create the world
	let mut world: world::World = world::World::default();
	world.start_world();

    while !window.should_close() {

		// Update the delta_time
		let new_time = glfw.get_time();
		world.time_manager.delta_time = new_time - world.time_manager.time_since_start;
		world.time_manager.time_since_start = new_time;
		// Update the world
		world.update_world();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    } 

	// When the window closes and we exit from the game
	world.stop_world();
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => {}
    }
}