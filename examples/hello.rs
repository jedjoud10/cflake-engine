use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_app_name("Hello World!")
        .execute();
}

// First function that gets executed when the engine starts
fn init(_: &mut World) {
    println!("Hello World!");
}
