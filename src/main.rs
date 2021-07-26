mod engine;
mod game;
use std::env;

use engine::core::window::setup_window;
extern crate gl;
// include the OpenGL type aliases
use gl::types::*;

fn main() {
	let args: Vec<String> = env::args().collect();
	// Check if we want to pack the resourcess
	let mut open_window = true;
	if args.len() > 0 {
		if args[1] == String::from("--pack-resources") {
			open_window = false;
			engine::resources::ResourceManager::pack_resources();
		} 
	} 
	if open_window {
		setup_window();
	}

}