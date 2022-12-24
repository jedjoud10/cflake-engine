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
    /*
    buffer1.write(&[1, 4], 0).unwrap();
    log::info!("{:?}", buffer1.as_slice());
    */

    // Create another new buffer
    /*
    let mut buffer2 = StorageBuffer::<u32>::with_capacity(
        &graphics,
        16,
        usage,
        &mut recorder
    ).unwrap();
    */

    // Copy the whole buffer1 into buffer2
    //buffer2.copy_from(&buffer1, &mut recorder).unwrap();
    //buffer2.extend_from_slice(&[220], &mut recorder).unwrap();

    // Submit to the GPU and wait for execution
    //graphics.submit(recorder).wait();
    
    // Read back the data
    /*
    let mut recorder = graphics.acquire();
    let data = buffer2.read_to_vec(&mut recorder).unwrap();
    graphics.submit(recorder).wait();
    */
    //dbg!(data);


    //let vert = assets.load::<VertexModule>("engine/shaders/basic.vert").unwrap();
    //let data = unsafe { translate_glsl_spirv(graphics.device(), "test", vert.source(), vert.kind()) };
}
