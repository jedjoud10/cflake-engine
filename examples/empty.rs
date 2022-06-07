use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::new().execute();
}

// Initialize the empty world
fn init(world: &mut World) {
}

/*
// Init the empty world
fn init(world: &mut World) {
    world.events.insert(run);
}

// Function that gets executed each frame
fn run(world: &mut World) {
    println!("Hello World, frame {}", world.time.count().unwrap())
}
*/
