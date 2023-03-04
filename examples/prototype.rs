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
    let mut buffer = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[1],
        BufferMode::Resizable,
        BufferUsage::WRITE | BufferUsage::READ
    ).unwrap();

    let mut dst = [0u32; 1];
    buffer.read(&mut dst, 0).unwrap();
    dbg!(dst);
    let mut dst = [0u32; 5];
    buffer.extend_from_slice(&[2, 3, 4, 5, 6]).unwrap();
    buffer.read(&mut dst, 0).unwrap();
    dbg!(dst);
    //panic!("Done here");
}

// Camera controller update executed every tick
fn update(world: &mut World) {
}
