use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_init(init)
        .execute();
}

fn init(world: &mut World) {
    world.insert::<Storage<ArrayBuffer<u32>>>(Storage::default());
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut storage = world.get_mut::<Storage<ArrayBuffer<u32>>>().unwrap();
    let mut buffer = ArrayBuffer::<u32>::from_slice(&mut ctx, &[0, 0, 0, 0], BufferMode::Resizable).unwrap();
    let mut mapped = buffer.map_mut().unwrap();
    let slice = mapped.as_slice_mut();
    slice[0] = 1;
    slice[3] = 1;
    drop(mapped);
    let mut mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    buffer.extend_from_slice(&[2, 3, 4, 5]);
    let mut mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);

    buffer.extend_from_slice(&[2, 3, 4, 5]);
    let mut mapped = buffer.map().unwrap();
    dbg!(mapped.as_slice());
    drop(mapped);


    buffer.extend_from_slice(vec![0; u16::MAX as usize].as_slice());

    let handle = storage.insert(buffer);
    unsafe { handle.increment_count() };
}