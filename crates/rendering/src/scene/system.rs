use crate::{ForwardRenderer, ForwardRendererRenderPass, Mesh, Basic, DefaultMaterialResources, Pipelines};
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

    // Create the scene renderer, pipeline manager, and  commonly used textures
    let renderer = ForwardRenderer::new(&graphics);
    let pipelines = Pipelines::new();
    
    // Add composites and basic storages
    drop(graphics);
    world.insert(renderer);
    world.insert(pipelines);
    world.insert(Storage::<Mesh>::default());
}

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let renderer = &mut *renderer;
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    
    // Get a presentable render target from the window
    let view = window.as_render_target().unwrap();

    // Extract the render pipelines
    let pipelines = pipelines.extract_pipelines();
    
    // Create a new command encoder
    let mut encoder = graphics.acquire();

    // Activate the render pass
    let mut render_pass = renderer.render_pass.begin(
        &mut encoder,
        view,
        ()
    ).unwrap();
    
    // Create the shared material resources1
    let default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        white: &renderer.white,
        black: &renderer.black,
        normal: &renderer.normal,
    };

    // This will iterate over each material pipeline and draw the scene
    for pipeline in pipelines.iter() {
        pipeline.render(world, &meshes, &default, &mut render_pass);
    }

    // Submit the encoder at the end
    drop(render_pass); 
    graphics.submit([encoder]);
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