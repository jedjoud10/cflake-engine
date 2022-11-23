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
    
    // Create a new async thread
    threadpool.for_each::<&[u32]>([0u32, 1, 2, 3].as_slice(), move |_| {
        // Create a uniform buffer
        let mut buffer = UniformBuffer::from_slice(
            &graphics,
            &[1i32, 2, 3],
            BufferMode::default()
        ).unwrap();

        let data = [1; 3];
        buffer.write(&data);
        /*
        // Write data back from uniform buffer
        buffer.write(&data);
        buffer.write(&data);
        buffer.write(&data);

        // Read data back from uniform buffer
        let mut data = [0; 3];
        buffer.read(&mut data);
        buffer.read(&mut data);
        buffer.read(&mut data);
        buffer.read(&mut data);
        */

        // Display data
        //dbg!(data);
    }, 1);    
}


// Executed each frame
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();

    if input.button(Button::P).pressed() {
        println!("{}", 1.0f32 / time.delta_f32());
    }
}