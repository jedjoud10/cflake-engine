use cflake_engine::*;

// An empty game window
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-empty", init)
}
// Init the empty world
fn init(_world: &mut World) {}
