use crate::{ForwardRenderer, ForwardRendererRenderPass, Mesh, Basic};
use graphics::{
    Graphics, Normalized, RenderPass,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA, Operation, StoreOp, LoadOp,
};
use std::{mem::ManuallyDrop, sync::Arc};
use utils::{Time, Storage};
use world::{post_user, user, System, World};

// Add the compositors and setup the world for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();

    // Create a new render pass with the appropriate color operations 
    let render_pass = ForwardRendererRenderPass::new(
        &graphics,
        Operation {
            load: LoadOp::Clear(vek::Vec4::broadcast(0)),
            store: StoreOp::Store,
        },
        (),
    ).unwrap();

    // Create the forward renderer wrapper that encapsulates the scene renderer
    let renderer = ForwardRenderer::new(&graphics, render_pass);
    drop(graphics);

    // Add the forward renderer and the basic storage utilities
    world.insert(renderer);
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Basic>::default());
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    
    // Get a presentable render target from the window
    let view = window.as_render_target().unwrap();
    
    // Activate the render pass
    renderer.render_pass.begin(
        view,
        ()
    );
}

// The rendering system will be resposible for iterating through the entities and displaying them
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::common);
    system.insert_update(update)
        .after(graphics::acquire)
        .before(graphics::present);
}