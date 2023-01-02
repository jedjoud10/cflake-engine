use std::time::Instant;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        //.set_frame_rate_limit(FrameRateLimit::Limited(30))
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let assets = world.get::<Assets>().unwrap();

    // Create a new buffer
    let mut buffer1 = StorageBuffer::<u32>::from_slice(
        &[1; 10],
        BufferMode::default(),
        BufferUsage::GpuToCpu,
    )
    .unwrap();

    log::info!("{:?}", buffer1.as_slice());
    buffer1.extend_from_slice(&[2; 5]).unwrap();
    log::info!("{:?}", buffer1.as_slice());

    let mut buffer2 = StorageBuffer::<u32>::from_slice(
        &[1; 10],
        BufferMode::default(),
        BufferUsage::GpuToCpu,
    )
    .unwrap();

    let mut texture = Texture2D::<RGBA<u32>>::from_texels(
        &[vek::Vec4::broadcast(u32::MAX); 64],
        vek::Extent2::broadcast(8),
        TextureMode::Dynamic,
        TextureUsage::Placeholder
    ).unwrap();

    type Test = Texture2D::<RGBA<Normalized<u8>>>;
    
    let mut texture = assets.load::<Test>("engine/textures/test.jpg").unwrap();

    /*
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let material_id = renderer.register::<Basic>(&assets);
    */
}
