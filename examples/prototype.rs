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

    let usage = BufferUsage {
        device_write: true,
        device_read: true,
        host_write: false,
        host_read: false,
    };

    // Create a new buffer
    let buffer1 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[69, 0, 0, 420],
        BufferMode::default(),
        usage,
        &mut recorder
    ).unwrap();

    // Create another new buffer
    let mut buffer2 = StorageBuffer::<u32>::from_slice(
        &graphics,
        &[0, 0, 0, 0],
        BufferMode::default(),
        usage,
        &mut recorder
    ).unwrap();

    // Copy the whole buffer1 into buffer2
    //buffer2.copy_from(&buffer1, &mut recorder).unwrap();
    
    // Submit to the GPU and wait for execution
    let submission = graphics.submit(recorder);
    let elapsed = submission.wait();

    // Create a temporary recorder to read back the data from buffer2
    //let mut recorder = graphics.acquire();
    //dbg!(buffer2.read_range_as_vec(.., &mut recorder).unwrap());
    //graphics.submit(recorder).wait();

    dbg!(elapsed);
}