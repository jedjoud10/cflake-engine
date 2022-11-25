use std::mem::size_of;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_update(update)
        .insert_init(init)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap().clone();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();    

    // Create a buffer in a new thread
    let array = [1, 2, 3, 4, 5u32];
    threadpool.for_each::<&[u32]>(&array, move |_| {
        // Create a uniform buffer
        let mut buffer = UniformBuffer::from_slice(
            &graphics.clone(),
            &[1i32, 2, 3],
            BufferMode::Dynamic,
            BufferUsage {
                hint_device_write: true,
                hint_device_read: true,
                host_write: true,
                host_read: true,
            },
        ).unwrap();

        buffer.write(&[3, 3, 3]).unwrap();

        let vec = buffer.read_to_vec().unwrap();
        dbg!(vec);
    }, 1);
    

    /*
    buffer.extend_from_slice(&[4]);

    */
}

// Executed each frame
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();

    if input.button(Button::P).pressed() {
        println!("{}", 1.0f32 / time.delta_f32());
    }
}
