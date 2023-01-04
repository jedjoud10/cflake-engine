use crate::{ForwardRenderer, WindowRenderTexture};
use graphics::{
    vk, Allocation, Graphics, Normalized, RenderPass, Swapchain,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA,
};
use std::mem::ManuallyDrop;
use utils::Time;
use world::{post_user, user, System, World};

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let render_targets =
        extract_swapchain_images_to_textures(&graphics);
    let extent = graphics.swapchain().extent();
    let format = [graphics.swapchain().format()];

    let attachment_image_info =
        vk::FramebufferAttachmentImageInfo::builder()
            .width(extent.w)
            .height(extent.h)
            .view_formats(&format)
            .layer_count(1)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_DST,
            );

    let render_pass = unsafe {
        RenderPass::new(
            &graphics,
            format[0],
            &[*attachment_image_info],
            vek::Rect {
                x: 0,
                y: 0,
                w: extent.w,
                h: extent.h,
            },
        )
    };
    drop(graphics);

    world.insert(ForwardRenderer::new(render_targets, render_pass));
}

// Create the texture wrappers from the swapchain
fn extract_swapchain_images_to_textures(
    graphics: &Graphics,
) -> Vec<WindowRenderTexture> {
    let images = graphics.swapchain().images();
    let dimensions = graphics.swapchain().extent();
    images
        .into_iter()
        .map(|(image, view)| unsafe {
            Texture2D::<BGRA<Normalized<u8>>>::from_raw_parts(
                graphics,
                image,
                view,
                Allocation::default(),
                dimensions,
                TextureUsage::Placeholder,
                TextureMode::Dynamic,
            )
        })
        .map(ManuallyDrop::new)
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
            swapchain.images();
        }
    }

    unsafe {
        // Acquire a new color image to render to
        let index = swapchain.acquire_next_image().unwrap();
        let texture =
            renderer.render_targets.get(index as usize).unwrap();
        let view = texture.view();
        let pipelines = renderer.extract_pipelines();

        // Check if we must recreate the swapchain
        if let None = swapchain.present(queue, device, index) {
            swapchain.resize(adapter, device, surface, window.size());
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
