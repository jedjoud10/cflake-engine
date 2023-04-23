use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        //.set_logging_level(LevelFilter::Trace)
        .insert_update(update)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();

    let buffer =
        Buffer::<i32, 0>::from_slice(&graphics, &[0, 1], BufferMode::Dynamic, BufferUsage::READ)
            .unwrap();

    drop(graphics);
    world.insert(buffer);
}

// Camera controller update executed every tick
fn update(world: &mut World) {
    let buffer = world.get::<Buffer::<i32, 0>>().unwrap();
    buffer.async_read(.., |data| {
        dbg!(data);
    }).unwrap();
}
