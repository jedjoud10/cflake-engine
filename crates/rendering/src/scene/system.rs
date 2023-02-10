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
    let render_pass = ForwardRendererRenderPass::new(
        &graphics,
    ).unwrap();
    let renderer = ForwardRenderer::new(&graphics, render_pass);
    drop(graphics);

    world.insert(renderer);
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Basic>::default());
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
}

// The rendering system will be resposible for iterating through the entities and displaying them
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::acquire);
    system.insert_update(update).after(post_user);
}