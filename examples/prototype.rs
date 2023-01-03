use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .set_frame_rate_limit(FrameRateLimit::Limited(4))
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let assets = world.get::<Assets>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let material_id = renderer.register::<Basic>(&assets);
}
