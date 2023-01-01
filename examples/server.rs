use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_app_name("Hello World!")
        .execute();
}

// Start hosting a new server
fn init(world: &mut World) {

}
