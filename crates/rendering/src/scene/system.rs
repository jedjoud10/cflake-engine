use crate::{ForwardRenderer, SwapchainFormat, ForwardRendererRenderPass, Mesh, Basic};
use graphics::{
    Graphics, Normalized, RenderPass,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA,
};
use std::{mem::ManuallyDrop, sync::Arc};
use utils::{Time, Storage};
use world::{post_user, user, System, World};

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let extent = graphics.swapchain().extent();
    let render_pass = ForwardRendererRenderPass::new(
        &graphics,
        extent,
    ).unwrap();
    let renderer = ForwardRenderer::new(&graphics, render_pass);
    drop(graphics);

    world.insert(renderer);
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Basic>::default());
}

// Called at the start of the system to acquire a new image to render to
fn acquire_image<'a>(graphics: &Graphics) -> (u32, graphics::RenderTarget<'a, BGRA<Normalized<u8>>>) {
    let swapchain = graphics.swapchain();
    let index = unsafe { swapchain.acquire_next_image() }.unwrap();
    let images = swapchain.images();
    let (image, view) = images[index as usize];
    let target = unsafe {
        graphics::RenderTarget::from_raw_parts(image, view)
    };
    (index, target)
}

// Present the newly rendered image onto the screen
fn present(graphics: &Graphics, index: u32, window: &Window, renderer: &mut ForwardRenderer) {
    let queue = graphics.queue();
    let device = graphics.device();
    let adapter = graphics.adapter();
    let surface = graphics.surface();
    let swapchain = graphics.swapchain();
    unsafe {
        if let None = swapchain.present(queue, device, index) {
            swapchain.resize(adapter, device, surface, window.size());
            renderer.render_pass.resize(window.size());
        }
    }
}

// This is called whenever we must recreate the swapchain and render pass
fn recreate(graphics: &Graphics, window: &Window, renderer: &mut ForwardRenderer) {
    let queue = graphics.queue();
    let device = graphics.device();
    let adapter = graphics.adapter();
    let surface = graphics.surface();
    let swapchain = graphics.swapchain();
    unsafe {
        swapchain.resize(adapter, device, surface, window.size());
        renderer.render_pass.resize(window.size());
    }
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();


    // Check if we must resize the swapchain
    if window.is_dirty() {
        recreate(&graphics, &window, &mut renderer);
    }

    // Acquire a new color image to render to
    let (index, target) = acquire_image(&graphics);

    // Extract the render pipelines
    let renderer = &mut *renderer;
    let pipelines = renderer.extract_pipelines();
    
    // Start the main render pass
    let render_pass = &mut renderer.render_pass;
    let mut rasterizer = render_pass.begin(
        target,
        (),
    ).unwrap();
    
    // Drop everything that is temporarily owned by the world
    drop(window);
    drop(graphics);
    for pipeline in pipelines {
        pipeline.render(world, &mut rasterizer);
    }
    unsafe { rasterizer.end().immediate_submit(); }

    // Finally, present the image onto the screen
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    present(&graphics, index, &window, renderer);
}

// The rendering system will be resposible for iterating through the entities and displaying them
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::system);
    system.insert_update(update).after(post_user);
}