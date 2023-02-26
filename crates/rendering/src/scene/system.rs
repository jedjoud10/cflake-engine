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
fn init_scene_renderers(world: &mut World) {
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
fn window_event(world: &mut World, event: &mut WindowEvent) {
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

// Update event that will set/update the main perspective camera
fn update_camera(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let window = world.get::<Window>().unwrap();

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
        let aspect = window.size().w as f32 / window.size().h as f32;
        camera.set_aspect_ratio(aspect);
        camera.update(location, rotation);

        // Convert the camera to uniform data
        let projection = (*camera.projection_matrix()).cols;
        let view = (*camera.view_matrix()).cols;
        let inverse_projection =
            (camera.projection_matrix().inverted()).cols;
        let inverse_view = (camera.view_matrix().inverted()).cols;

        // Create the struct that contains the UBO data
        let data = CameraUniform {
            projection,
            inverse_projection,
            view,
            inverse_view,
            position: (*location).with_w(0.0),
            forward: rotation.forward().with_w(0.0),
            right: rotation.right().with_w(0.0),
            up: rotation.up().with_w(0.0),
        };

        // Fill the camera UBO with the proper data
        renderer.camera_buffer.write(&[data], 0).unwrap();
    } else {
        // Set the main camera if we did not find one
        let next = ecs.find::<(
            &Camera,
            &ecs::Position,
            &ecs::Rotation,
            &ecs::Entity,
        )>();
        if let Some((_, _, _, entity)) = next {
            renderer.main_camera = Some(*entity);
        }
    }
}

// Update the global mesh matrices of objects that have been modified
fn update_matrices(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    use ecs::*;

    // Filter the objects that have changed only
    let f1 = modified::<Position>();
    let f2 = modified::<Rotation>();
    let f3 = modified::<Scale>();
    let f4 = added::<Renderer>();
    //let filter = f1 | f2 | f3 | f4;
    let query = scene
        .query_mut_with::<(
            &mut Renderer,
            Option<&ecs::Position>,
            Option<&ecs::Rotation>,
            Option<&ecs::Scale>,
        )>(f1);
        
    // Update the matrices of objects that might contain location, rotation, or scale
    for (renderer, location, rotation, scale) in query {
        let mut matrix = vek::Mat4::<f32>::identity();
        matrix = location.map_or(matrix, |l| matrix * vek::Mat4::from(l));
        matrix *= rotation.map_or(matrix, |r| matrix * vek::Mat4::from(r));
        matrix *= scale.map_or(matrix, |s| matrix * vek::Mat4::from(s));
        renderer.matrix = matrix;
    }
}

// Clear the window and render the entities to the texture
fn render_update(world: &mut World) {
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

    // Begin the render pass
    let mut render_pass =
        renderer.render_pass.begin(&mut encoder, color, depth).unwrap();

    // Skip if we don't have a camera to draw with
    if renderer.main_camera.is_none() {
        log::warn!("No active camera to draw with!");
        drop(render_pass);
        graphics.submit([encoder]);
        return;
    }

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

// Inserts the final render pass renderer and post processing params
fn init_display_pp(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let pp = PostProcess::new(&graphics, &mut assets);
    drop(graphics);
    drop(assets);
    world.insert(pp);
}

// Displays the rendered scene texture to the actual window texture (post-processing pass)
fn display(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let renderer = world.get::<ForwardRenderer>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let mut postprocess = world.get_mut::<PostProcess>().unwrap();
    let postprocess = &mut *postprocess;

    // Get textures, pipelines, and encoder
    let src = &renderer.color_texture;
    let dst = window.as_render_target().unwrap();
    let mut encoder = graphics.acquire();

    // Begin the render pass
    let mut render_pass =
        postprocess.render_pass.begin(&mut encoder, dst, ()).unwrap();

    // Bind the graphics pipeline
    let mut active = render_pass.bind_pipeline(&postprocess.pipeline);

    // Set the required shader uniforms
    active.set_bind_group(0, |group| {
        group.set_texture("color_map", src).unwrap();
        group.set_sampler("color_map_sampler", src.sampler()).unwrap();
        group.set_buffer("window", &renderer.window_buffer).unwrap();
    });

    // Draw 6 vertices (2 tris)
    active.draw(0..6, 0..1);

    drop(render_pass);

    // Submit the encoder at the end
    graphics.submit([encoder]);
} 

// The rendering system will be resposible for iterating through the entities and rendering them to the backbuffer texture
pub fn rendering_system(system: &mut System) {
    system
        .insert_init(init_scene_renderers)
        .before(user)
        .after(graphics::common);
    system
        .insert_update(render_update)
        .after(graphics::acquire)
        .before(graphics::present);
    system
        .insert_window(window_event)
        .after(graphics::common)
        .before(user);
}

// The display system will be responsible for displaying the renderered scene textures to the scene
pub fn display_system(system: &mut System) {
    system.insert_init(init_display_pp)
        .before(user)
        .after(graphics::common);
    system.insert_update(display)
        .after(rendering_system)
        .after(graphics::acquire)
        .before(graphics::present);
}

// The camera system will be responsible for updating the camera UBO and matrices
pub fn camera_system(system: &mut System) {
    system.insert_update(update_camera)
        .before(rendering_system);
}

// The matrix system will be responsible for updating the matrices of the renderer
pub fn matrix_system(system: &mut System) {
    system.insert_update(update_matrices)
        .before(rendering_system)
        .after(post_user);
}