use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_app_name("Hello World!")
        .execute();
}

// Start a client and connect to the machine hosted server
fn init(world: &mut World) {
    world.insert(Client::connect("localhost:25565"));
}
