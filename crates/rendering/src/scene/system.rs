use graphics::{Graphics, Window, vk};
use world::{post_user, System, World};

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let queue = graphics.queue();
    let device = graphics.device();
    let adapter = graphics.adapter();
    let surface = graphics.surface();
    let swapchain = graphics.swapchain();

    unsafe {
        let img = swapchain.acquire_next_image().unwrap();

        // Image whole subresource range (TODO: Implement mipmapping
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        // Convert the image layout to PRESENT
        let image_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access_mask(vk::AccessFlags::MEMORY_READ)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .subresource_range(*subresource_range)
            .image(img.1);

        let mut recorder = queue.acquire(device);
        recorder.cmd_image_memory_barrier(*image_barrier);
        recorder.immediate_submit();
        swapchain.present(queue, img).unwrap();
        /*
        let submission = queue.submit(recorder).wait();
        log::info!("{:?}", submission);
        */
    }
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
