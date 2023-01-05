use crate::{ForwardRenderer, SwapchainTexture, SwapchainFormat, ForwardRendererRenderPass};
use graphics::{
    vk, Graphics, Normalized, RenderPass, Swapchain,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA, Adapter, Device, gpu_allocator::vulkan::Allocation,
};
use std::{mem::ManuallyDrop, sync::Arc};
use utils::Time;
use world::{post_user, user, System, World};

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let render_targets =
        extract_swapchain_images_to_textures(&graphics);
    let extent = graphics.swapchain().extent();
    let render_pass = ForwardRendererRenderPass::new(
        &graphics,
        extent,
    ).unwrap();
    drop(graphics);

    world.insert(ForwardRenderer::new(render_targets, render_pass));
}

// Create the texture wrappers from the swapchain
fn extract_swapchain_images_to_textures(
    graphics: &Graphics,
) -> Vec<SwapchainTexture> {
    let images = graphics.swapchain().images();
    let dimensions = graphics.swapchain().extent();

    

    images
        .into_iter()
        .map(|(image, view)| unsafe {
            Texture2D::<SwapchainFormat>::from_raw_parts(
                graphics,
                image,
                view,
                Allocation::default(),
                dimensions,
                TextureUsage::Placeholder,
                TextureMode::Dynamic,
            )
        })
        .collect::<Vec<_>>()
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
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
            renderer.render_pass.resize(window.size());
            renderer.render_targets =
                extract_swapchain_images_to_textures(&graphics);
        }
    }

    unsafe {
        // Acquire a new color image to render to
        let index = swapchain.acquire_next_image().unwrap();
        let renderer = &mut *renderer;
        let pipelines = renderer.extract_pipelines();
        let targets = &mut renderer.render_targets;
        let render_pass = &mut renderer.render_pass;
        let texture = targets.get_mut(index as usize).unwrap();
        let texture = &mut *texture;

        // Start the main render passd
        device.wait();
        let mut rasterizer = render_pass.begin(texture, (), window.viewport()).unwrap();
        
        // Render the surfaces
        for pipeline in pipelines {
            pipeline.render(&mut rasterizer);
        }
        rasterizer.end().immediate_submit();

        // Check if we must recreate the swapchain
        if let None = swapchain.present(queue, device, index) {
            swapchain.resize(adapter, device, surface, window.size());
            renderer.render_pass.resize(window.size());
            renderer.render_targets =
                extract_swapchain_images_to_textures(&graphics);
        }
    }
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::system);
    system.insert_update(update).after(post_user);
}
