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
    
    // Create a recorder to record commands
    let mut recorder = graphics.acquire();

    // Create a new buffer
    let buffer1 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[69, 0, 0, 420],
        BufferMode::default(),
        BufferUsage::default(),
        &mut recorder
    ).unwrap();

    // Create another new buffer
    let mut buffer2 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[0, 0, 0, 0],
        BufferMode::default(),
        BufferUsage::default(),
        &mut recorder
    ).unwrap();

    // Create another new buffer
    let mut buffer2 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[0, 0, 0, 0],
        BufferMode::default(),
        BufferUsage::default(),
        &mut recorder
    ).unwrap();

    // Copy the whole buffer1 into buffer2
    buffer2.copy_from(&buffer1, &mut recorder).unwrap();
    let submission = graphics.submit(recorder);
    let elapsed = submission.wait();
    let mut recorder = graphics.acquire();
    let vec = buffer2.read_range_as_vec(.., &mut recorder).unwrap();
    dbg!(vec);
    graphics.submit(recorder).wait();

    // Submit to the GPU and wait for execution
    dbg!(elapsed);
}