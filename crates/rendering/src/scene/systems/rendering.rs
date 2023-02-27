use crate::{
    AlbedoMap, Sky, Camera, CameraUniform,
    DefaultMaterialResources, ForwardRenderer,
    SceneRenderPass, Mesh, NormalMap, Pipelines, Renderer, Basic, PostProcess, WindowUniform,
};
use assets::Assets;
use ecs::Scene;
use graphics::{
    Graphics, LoadOp, Normalized, Operation, RenderPass, StoreOp,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA,
};
use std::{mem::ManuallyDrop, sync::Arc};
use utils::{Storage, Time};
use world::{post_user, user, System, World, WindowEvent};


// Add the scene resources and setup for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create the scene renderer, pipeline manager, and post-processing
    let renderer = ForwardRenderer::new(&graphics, &mut assets, window.size());
    let pipelines = Pipelines::new();

    // Drop fetched resources
    drop(graphics);
    drop(window);
    drop(assets);

    // Add composites and basic storages
    world.insert(renderer);
    world.insert(pipelines);

    // Add common storages
    world.insert(Storage::<Mesh>::default());

    // Add the storages that contain the materials and their resources
    world.insert(Storage::<Basic>::default());
    world.insert(Storage::<Sky>::default());
    world.insert(Storage::<AlbedoMap>::default());
    world.insert(Storage::<NormalMap>::default());
}


// Handle window resizing the depth texture
fn event(world: &mut World, event: &mut WindowEvent) {
    match event {
        // Window has been resized
        WindowEvent::Resized(size) => {
            // Check if the size is valid
            if size.height == 0 || size.height == 0 {
                return;
            }

            // Handle resizing the depth texture
            let size = vek::Extent2::new(size.width, size.height);
            let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();

            // Resize the color and depth texture
            renderer.depth_texture.resize(size).unwrap();
            renderer.color_texture.resize(size).unwrap();

            // Update the uniform
            renderer.window_buffer.write(&[
                WindowUniform {
                    width: size.w,
                    height: size.h,
                }
            ], 0).unwrap();
        }

        _ => (),
    }
}

// Clear the window and render the entities to the texture
fn render(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let renderer = &mut *renderer;
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();

    // Get textures, pipelines, and encoder
    let color = renderer.color_texture.as_render_target().unwrap();
    let depth = renderer.depth_texture.as_render_target().unwrap();
    let pipelines = pipelines.extract_pipelines();
    let mut encoder = graphics.acquire();

    // Skip if we don't have a camera to draw with
    if renderer.main_camera.is_none() {
        log::warn!("No active camera to draw with!");
        return;
    }

    // Begin the render pass
    let mut render_pass =
        renderer.render_pass.begin(&mut encoder, color, depth).unwrap();

    // Create the shared material resources1
    let default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        white: &renderer.white,
        black: &renderer.black,
        normal: &renderer.normal,
        sky_gradient: &renderer.sky_gradient,
    };

    // This will iterate over each material pipeline and draw the scene
    for pipeline in pipelines.iter() {
        pipeline.render(world, &meshes, &default, &mut render_pass);
    }

    drop(render_pass);

    // Submit the encoder at the end
    graphics.submit([encoder]);
} 

// The rendering system will be resposible for iterating through the entities and rendering them to the backbuffer texture
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::common);
    system
        .insert_update(render)
        .after(graphics::acquire)
        .before(graphics::present);
    system
        .insert_window(event)
        .after(graphics::common)
        .before(user);
}
