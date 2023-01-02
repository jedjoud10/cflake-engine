use std::mem::ManuallyDrop;
use graphics::{Graphics, Window, vk, Texture2D, Texture, Allocation, RenderPass, BGRA, Normalized, TextureUsage, TextureMode, Swapchain};
use utils::Time;
use world::{post_user, System, World, user};
use crate::ForwardRenderer;

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    let graphics = Graphics::global();
    let render_targets = swapchain_images_to_textures(graphics.swapchain());
    world.insert(ForwardRenderer::new(render_targets))
}

// Create the texture wrappers from the swapchain
fn swapchain_images_to_textures(swapchain: &Swapchain)-> Vec<ManuallyDrop<Texture2D::<BGRA<Normalized<u8>>>>> {
    let images = swapchain.images();
    let dimensions = swapchain.extent();
    images.into_iter().map(|(image, view)| unsafe {
        Texture2D::<BGRA<Normalized<u8>>>::from_raw_parts(
            image,
            view,
            Allocation::default(),
            dimensions,
            TextureUsage::Placeholder,
            TextureMode::Dynamic,
        )
    }).map(ManuallyDrop::new).collect::<Vec<_>>()
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

    // Check if we must resize the swapchain
    if window.is_dirty() {
        unsafe {
            swapchain.resize(adapter, device, surface, window.size());
            swapchain.images();
        }
    }

    unsafe {
        // Acquire a new color image to render to
        let index = swapchain.acquire_next_image().unwrap();
        let texture = renderer.render_targets.get(index as usize).unwrap();



        // Check if we must recreate the swapchain
        if let None = swapchain.present(queue, device, index) {
            swapchain.resize(adapter, device, surface, window.size());
            renderer.render_targets = swapchain_images_to_textures(swapchain);
        }
    }
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system.insert_init(init).before(user).after(graphics::system);
    system.insert_update(update).after(post_user);
}
