use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine server example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Start hosting a new server
fn init(world: &mut World) {
}

// Receive the packets coming from the clients
fn update(world: &mut World) {
}
