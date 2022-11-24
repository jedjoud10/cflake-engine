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

    // Create a uniform buffer
    let mut buffer = UniformBuffer::from_slice(
        &graphics,
        &[1i32, 2, 3],
        BufferMode::Resizable,
    );

    /*
    buffer.extend_from_slice(&[4]);

    let vec = buffer.read_to_vec();
    dbg!(vec);
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
