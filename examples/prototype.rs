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

    let buffer = Buffer::<i32, 0>::from_slice(
        &graphics,
        &[0, 1],
        BufferMode::Dynamic,
        BufferUsage::READ,
    )
    .unwrap();
    graphics.submit(true);

    let instant = std::time::Instant::now();
    let view = buffer.as_view(..).unwrap();
    dbg!(&view.as_slice());
    dbg!(instant.elapsed());

    let instant = std::time::Instant::now();
    let view = buffer.as_view(..).unwrap();
    dbg!(&view.as_slice());
    dbg!(instant.elapsed());
}

// Camera controller update executed every tick
fn update(_world: &mut World) {}
