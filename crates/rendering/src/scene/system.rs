use graphics::{Graphics, Window, vk, Texture2D, Texture, Allocation, RenderPass};
use utils::Time;
use world::{post_user, System, World, user};

use crate::ForwardRenderer;

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    world.insert(ForwardRenderer::new(renderpass))
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = Graphics::global();
    let window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let time = world.get::<Time>().unwrap();
    let queue = graphics.queue();
    let device = graphics.device();
    let adapter = graphics.adapter();
    let surface = graphics.surface();
    let swapchain = graphics.swapchain();

    unsafe {
        // Acquire a new color image to render to
        let (index, image) = swapchain.acquire_next_image().unwrap();

        // Image whole subresource range (TODO: Implement mipmapping
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        let image_barrier1 = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_access_mask(vk::AccessFlags::MEMORY_READ)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .subresource_range(*subresource_range)
            .image(image);

        let image_barrier2 = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .subresource_range(*subresource_range)
            .image(image);

        let value = time.since_startup().as_secs_f32().sin() * 0.5 + 0.5;

        let color = vk::ClearColorValue {
            float32: [value; 4]
        };

        let mut recorder = queue.acquire(device);
        recorder.cmd_image_memory_barrier(*image_barrier1);
        recorder.cmd_clear_image(image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, color, &[*subresource_range]);
        recorder.cmd_image_memory_barrier(*image_barrier2);
        recorder.immediate_submit();

        // Check if we must recreate the swapchain
        if let None = swapchain.present(queue, (index, image)) {
            swapchain.resize(adapter, device, surface, window.size());
        }
    }
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system.insert_init(init).before(user).after(graphics::system);
    system.insert_update(update).after(post_user);
}
