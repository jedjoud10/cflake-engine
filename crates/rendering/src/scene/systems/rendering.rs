use crate::{
    AlbedoMap, Basic,
    DefaultMaterialResources, ForwardRenderer, Mesh, NormalMap,
    Pipelines, Sky,
    WindowUniform, ShadowMapping,
};
use assets::Assets;

use graphics::{
    Graphics,
    Texture, Window,
};

use utils::{Storage, Time};
use world::{user, System, WindowEvent, World};

// Add the scene resources and setup for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create the scene renderer, pipeline manager
    let renderer =
        ForwardRenderer::new(&graphics, &mut assets, window.size());
    let pipelines = Pipelines::new();

    // Create a nice shadow map
    let shadowmap = ShadowMapping::new(
        10f32,
        40f32,
        4096,
        &graphics,
        &mut assets
    );

    // Drop fetched resources
    drop(graphics);
    drop(window);
    drop(assets);

    // Add composites and basic storages
    world.insert(renderer);
    world.insert(pipelines);
    world.insert(shadowmap);

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
            let mut renderer =
                world.get_mut::<ForwardRenderer>().unwrap();

            // Resize the color and depth texture
            renderer.depth_texture.resize(size).unwrap();
            renderer.color_texture.resize(size).unwrap();

            // Update the uniform
            renderer
                .window_buffer
                .write(
                    &[WindowUniform {
                        width: size.w,
                        height: size.h,
                    }],
                    0,
                )
                .unwrap();
        }

        _ => (),
    }
}

// Clear the window and render the entities to the texture
fn render(world: &mut World) {
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let mut _shadowmap = world.get_mut::<ShadowMapping>().unwrap();
    let renderer = &mut *renderer;
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let time = world.get::<Time>().unwrap();

    let pipelines = pipelines.extract_pipelines();

    // Skip if we don't have a camera to draw with
    if renderer.main_camera.is_none() {
        log::warn!("No active camera to draw with!");
        return;
    }

    // Create the shared material resources
    let default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        white: &renderer.white,
        black: &renderer.black,
        normal: &renderer.normal,
        sky_gradient: &renderer.sky_gradient,
    };

    // Begin the scene shadow map render pass
    let shadowmap = &mut *_shadowmap;
    shadowmap.update(vek::Quaternion::rotation_x(time.elapsed().as_secs_f32() * 0.2));
    let depth = shadowmap.depth_tex.as_render_target().unwrap();
    let mut render_pass = shadowmap
        .render_pass.begin((), depth).unwrap();
    let mut active = render_pass.bind_pipeline(&shadowmap.pipeline);
    active.set_bind_group(0, |group| {
        group.set_uniform_buffer("shadow", &shadowmap.buffer).unwrap();
    });

    // Render the shadows first (fuck you)
    for pipeline in pipelines.iter() {
        pipeline.prerender(world, &meshes, &default, &mut active);
    }
    drop(active);
    drop(render_pass);
    drop(shadowmap);
    drop(_shadowmap);

    // Begin the scene color render pass
    let color = renderer.color_texture.as_render_target().unwrap();
    let depth = renderer.depth_texture.as_render_target().unwrap();
    let mut render_pass =
        renderer.render_pass.begin(color, depth).unwrap();

    // This will iterate over each material pipeline and draw the scene
    for pipeline in pipelines.iter() {
        pipeline.render(world, &meshes, &default, &mut render_pass);
    }

    drop(render_pass);
}

// The rendering system will be resposible for iterating through the entities and rendering them to the backbuffer texture
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(assets::system)
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
