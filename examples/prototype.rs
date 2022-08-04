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
    world.insert::<Storage<ArrayBuffer<MyData>>>(Storage::default());
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut storage = world.get_mut::<Storage<ArrayBuffer<MyData>>>().unwrap();
    let mut buffer = ArrayBuffer::<MyData>::from_slice(&mut ctx, &[MyData::default(), MyData::default(), MyData::default(), MyData::default()], BufferMode::Resizable).unwrap();
    let mut mapped = buffer.map_mut().unwrap();
    let slice = mapped.as_slice_mut();
    slice[0].humidity = 1.0;
    slice[3].humidity = 1.0;
    drop(mapped);
    let mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    buffer.extend_from_slice(&[MyData::default()]);
    let mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    buffer.extend_from_slice(&[MyData::default()]);
    let mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    let handle = storage.insert(buffer);
    unsafe { handle.increment_count() };
}