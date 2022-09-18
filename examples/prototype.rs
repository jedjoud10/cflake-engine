

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_init(init)
        .execute();
}

#[derive(Default, Clone, Copy, Debug)]
struct MyData {
    temperature: f32,
    humidity: f32,
    pressure: f32,
}

fn init(world: &mut World) {
    let _ctx = world.get_mut::<Context>().unwrap();
}
