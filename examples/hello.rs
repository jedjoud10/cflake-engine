use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine hello example")
        .set_user_assets(user_assets!("/examples/assets/"))
        .insert_init(init)
        .execute();
}

// First function that gets executed when the engine starts
fn init(_: &mut World) {
    println!("Hello World!");
}
