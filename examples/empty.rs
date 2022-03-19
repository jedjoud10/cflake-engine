use cflake_engine::*;

// An empty game window
fn main() {
    cflake_engine::start("cflake-examples", "empty", init, |_| {})
}
// Init the empty world
fn init(_world: &mut World) {}
