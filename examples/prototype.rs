use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .insert_update(update)
        .set_frame_rate_limit(FrameRateLimit::Limited(60))
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let material_id = renderer.register::<Basic>(&graphics, &assets);
}

fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let input = world.get::<Input>().unwrap();

    if input.get_button(Button::F5).pressed() {
        dbg!(time.average_fps());
    }
}
