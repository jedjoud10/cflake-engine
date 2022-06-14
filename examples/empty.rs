use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::new().insert_startup(init).execute();
}

// Initialize the empty world
fn init(_world: &mut World) {
    println!("Start!")
}