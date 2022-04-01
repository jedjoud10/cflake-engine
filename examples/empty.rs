use cflake_engine::{*};

// An empty game window
fn main() {
    cflake_engine::start("cflake-examples", "empty", init)
}

// Init the empty world
fn init(world: &mut World) {
    world.systems.insert(run);
}

// Function that gets executed each frame
fn run(world: &mut World) {
    println!("Hello World, frame {}", world.time.count().unwrap())
}