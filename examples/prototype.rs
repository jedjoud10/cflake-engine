use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();

    // Load a vertex shader
    let vert = assets.load::<VertexModule>("engine/shaders/basic.vert").unwrap();
    let processor = Processor::new(vert, &assets);
    let compiled = Compiled::compile(&graphics, processor.process());
}
