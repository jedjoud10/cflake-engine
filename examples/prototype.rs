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

    // Create a new buffer
    let mut buffer1 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[1; 10],
        BufferMode::default(),
        BufferUsage::GpuToCpu,
    ).unwrap();

    log::info!("{:?}", buffer1.as_slice());
    buffer1.extend_from_slice(&[2; 5]).unwrap();
    log::info!("{:?}", buffer1.as_slice());

    let mut buffer2 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[1; 10],
        BufferMode::default(),
        BufferUsage::GpuToCpu,
    ).unwrap();
}
