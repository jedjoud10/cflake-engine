//#![windows_subsystem = "windows"]
mod engine;
mod game;
use std::env;

use engine::core::window::setup_window;
extern crate gl;
// include the OpenGL type aliases

fn main() {
    setup_window();    
}
