use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine client example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Start a client and connect to the machine hosted server
fn init(world: &mut World) {
}

// Send a packet to the server everytime we press the A key
fn update(world: &mut World) {
}
