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
        println!("{}", 1.0f32 / time.delta_f32());
    }
}
