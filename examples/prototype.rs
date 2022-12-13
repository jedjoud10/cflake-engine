use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_update(update)
        .insert_init(init)
        .set_frame_rate_limit(FrameRateLimit::VSync)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap().clone();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();

    unsafe {
        let device = graphics.device();
        let queue = graphics.queue();
        let usage = cflake_engine::graphics::vk::BufferUsageFlags::TRANSFER_SRC | 
        cflake_engine::graphics::vk::BufferUsageFlags::TRANSFER_DST | cflake_engine::graphics::vk::BufferUsageFlags::STORAGE_BUFFER;
        let copy = *cflake_engine::graphics::vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(4);
        let copy2 = *cflake_engine::graphics::vk::BufferCopy::builder()
            .src_offset(2)
            .dst_offset(2)
            .size(2);
        let loc = cflake_engine::graphics::MemoryLocation::CpuToGpu;

        let (buffer, mut allocation) = device.create_buffer(4, usage, loc, queue);
        let ptr = allocation.mapped_slice_mut().unwrap();
        ptr[0] = 5;
        let (buffer2, allocation2) = device.create_buffer(4, usage, loc, queue);
        let (buffer3, allocation3) = device.create_buffer(4, usage, loc, queue);
        
        let mut recorder = graphics.acquire();
        recorder.cmd_clear_buffer(buffer, 0, 4);
        let submission = graphics.submit(recorder);
        submission.flush(device);
        
        let data = allocation2.mapped_slice().unwrap()[0];

        device.destroy_buffer(buffer, allocation);
        device.destroy_buffer(buffer2, allocation2);
        device.destroy_buffer(buffer3, allocation3);
    }

}

// Executed each frame
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();

    if input.get_button(Button::P).pressed() {
        println!("{}", time.average_fps());
    }
}
