use cflake_engine::prelude::*;

fn main() {
    App::default()
        .insert_init(init)
        .execute();
}

fn init(world: &mut World) {
    println!("Hello world!");
}