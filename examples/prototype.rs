use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_init(init)
        .execute();
}

#[derive(Default, Clone, Copy, Debug)]
struct MyData {
    temperature: f32,
    humidity: f32,
    pressure: f32,
}

fn init(world: &mut World) {
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut fence = Fence::new(&mut ctx);
    fence.start();
    let mut buffer = ArrayBuffer::<MyData>::from_slice(
        &mut ctx,
        vec![MyData::default(); 4096 * 4].as_slice(),
        BufferMode::Dynamic {
            map_write: true,
            map_read: true,
            persistent: true,
            client: true,
        },
    )
    .unwrap();
    let mut buffer2 = ArrayBuffer::<MyData>::from_slice(
        &mut ctx,
        vec![MyData::default(); 4096 * 4].as_slice(),
        BufferMode::Resizable,
    )
    .unwrap();
    let i = Instant::now();
    buffer2.copy_from(&buffer);
    println!("{}", i.elapsed().as_millis());
    let mut mapped = buffer.view_mut().unwrap();
    let slice = mapped.as_mut_slice();
    slice[0].humidity = 1.0;
    slice[3].humidity = 1.0;
    drop(mapped);

    let mapped = buffer.view().unwrap();
    let mapped2 = buffer.view().unwrap();
    let vec = buffer.read_to_vec();
    fence.stop();

    let duration = Instant::now();
    while !fence.signaled() {
        println!("{}", duration.elapsed().as_millis());
    }
    //dbg!(vec);
    //dbg!(mapped.as_slice());
    drop(mapped);

    /*
    buffer.extend_from_slice(&[MyData::default()]);
    let mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    buffer.extend_from_slice(&[MyData::default()]);
    let mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);
    */
}
