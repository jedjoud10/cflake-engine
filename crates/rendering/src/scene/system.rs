use crate::{
    AlbedoMap, Basic, DefaultMaterialResources, ForwardRenderer,
    ForwardRendererRenderPass, Mesh, NormalMap, Pipelines, Camera,
};
use ecs::Scene;
use graphics::{
    Graphics, LoadOp, Normalized, Operation, RenderPass, StoreOp,
    Texture, Texture2D, TextureMode, TextureUsage, Window, BGRA,
};
use std::{mem::ManuallyDrop, sync::Arc};
use utils::{Storage, Time};
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

    // TODO: Handle this automatically
    world.insert(Storage::<Basic>::default());
    world.insert(Storage::<AlbedoMap>::default());
    world.insert(Storage::<NormalMap>::default());
}

// Update event that will set/update the main perspective camera
fn update_camera(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = renderer.main_camera {
        // Disable the entity in the resource if it got removed
        let mut entry = if let Some(entry) = ecs.entry_mut(entity) {
            entry
        } else {
            renderer.main_camera = None;
            return;
        };

        // Fetch it's components,and update them
        let (camera, location, rotation) = entry
            .as_query_mut::<(&mut Camera, &ecs::Position, &ecs::Rotation)>()
            .unwrap();
        camera.update(location, rotation);

        // Fill the camera UBO with the proper data
        renderer.camera_buffer.write(&[camera.as_uniform_data()], 0).unwrap();
    } else {
        // Set the main camera if we did not find one
        let next = ecs
            .find::<(&Camera, &ecs::Position, &ecs::Rotation, &ecs::Entity)>();
        if let Some((_, _, _, entity)) = next {
            renderer.main_camera = Some(*entity);
        }
    }
}

// Clear the window and render the entities
fn render_update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let renderer = &mut *renderer;
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();

    // Skip if we don't have a camera to draw with
    if renderer.main_camera.is_none() {
        log::warn!("No active camera to draw with!");
        return;
    }

    // Get textures, pipelines, and encoder
    let view = window.as_render_target().unwrap();
    let pipelines = pipelines.extract_pipelines();
    let mut encoder = graphics.acquire();

    // Activate the render pass
    let mut render_pass =
        renderer.render_pass.begin(&mut encoder, view, ()).unwrap();

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

    drop(render_pass);

    // Submit the encoder at the end
    graphics.submit([encoder]);
}

// The rendering system will be resposible for iterating through the entities and displaying them
pub fn rendering_system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(graphics::common);
    system
        .insert_update(render_update)
        .after(graphics::acquire)
        .before(graphics::present);
}

// The camera system will be responsible for updating the camera UBO and matrices
pub fn camera_system(system: &mut System) {
    system
        .insert_update(update_camera)
        .before(rendering_system);
}