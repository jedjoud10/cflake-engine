use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let buffer = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[],
        BufferMode::Resizable,
        BufferUsage::COPY_SRC
    ).unwrap();
}

// Camera controller update executed every tick
fn update(world: &mut World) {
}
