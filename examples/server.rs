use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_app_name("Server!")
        .execute();
}

// Start hosting a new server
fn init(world: &mut World) {
    world.insert(NetworkedSession::host(8080).unwrap());
}
