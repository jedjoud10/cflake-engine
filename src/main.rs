mod engine;
mod game;
use engine::core::window::setup_window;
extern crate gl;
// include the OpenGL type aliases
use gl::types::*;

fn main() {
	setup_window();
}