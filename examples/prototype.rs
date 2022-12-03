use std::mem::size_of;

use cflake_engine::prelude::{*, vulkano::sync};

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

    threadpool.execute(move || {
        // Create a command buffer just for this buffer
        let mut recorder = graphics.acquire();

        // Create the buffer usage
        let usage = BufferUsage {
            hint_device_write: true,
            hint_device_read: false,
            permission_host_write: false,
            permission_host_read: true,
        };

        // Create a simple buffer
        let mut buffer = UniformBuffer::from_slice(
            &graphics,
            &[0, 1, 2],
            Default::default(),
            usage,
            &mut recorder,
        ).unwrap();

        
        let mut dst = [0; 3];
        buffer.read_range(&mut dst, 0..3).unwrap();
        dbg!(dst);

        buffer.write_range(&[2, 3, 4], 0..3).unwrap();

        buffer.read_range(&mut dst, 0..3).unwrap();
        dbg!(dst);

        graphics.submit(recorder);

        buffer.read_range(&mut dst, 0..3).unwrap();
        dbg!(dst);
    });
    

    /*
    let array = (0..1).into_iter().collect::<Vec<_>>();
    threadpool.for_each::<&[u32]>(
        &array,
        move |_| {
            log::warn!(
                "Executing on thread {:?}",
                std::thread::current().name()
            );
            
            // Create a command buffer just for this buffer
            let builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
                graphics.cmd_buffer_allocator(),
                graphics.queue().queue_family_index(),
                vulkano::command_buffer::CommandBufferUsage::SimultaneousUse
            ).unwrap();
        },
        1,
    );
    */

    //std::thread::sleep(std::time::Duration::from_secs(10));
}

// Executed each frame
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();

    if input.get_button(Button::P).pressed() {
        println!("{}", time.average_fps());
    }
}
