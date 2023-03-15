use crate::{
    AlbedoMap, Basic, DefaultMaterialResources, DirectionalLight,
    ForwardRenderer, Mesh, NormalMap, Pipelines, Renderer,
    SceneUniform, ShadowMapping, Sky, WindowUniform,
};
use assets::Assets;

use ecs::{Rotation, Scene};
use graphics::{Graphics, Texture, Window};

use utils::{Storage, Time};
use world::{user, System, WindowEvent, World};

// Add the scene resources and setup for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut albedo_maps = Storage::<AlbedoMap>::default();
    let mut normal_maps = Storage::<NormalMap>::default();

    // Create the scene renderer, pipeline manager
    let renderer = ForwardRenderer::new(
        &graphics,
        &mut assets,
        window.size(),
        &mut albedo_maps,
        &mut normal_maps,
    );
    let pipelines = Pipelines::new();

    // Create a nice shadow map
    let shadowmap = ShadowMapping::new(
        20f32,
        100f32,
        512,
        &graphics,
        &mut assets,
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
    world.insert(albedo_maps);
    world.insert(normal_maps);
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
    let scene = world.get::<Scene>().unwrap();
    let _time = world.get::<Time>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
    let normal_maps = world.get::<Storage<NormalMap>>().unwrap();

    let pipelines = pipelines.extract_pipelines();

    // Skip if we don't have a camera to draw with
    if renderer.main_camera.is_none() {
        log::warn!("No active camera to draw with!");
        return;
    }

    // Skip if we don't have a light to draw with
    if renderer.main_directional_light.is_none() {
        log::warn!("No directional light to draw with!");
        return;
    }

    // Get the directioanl light and rotation of the light
    let id = renderer.main_directional_light.unwrap();
    let entity = scene.entry(id).unwrap();
    let light = entity.get::<DirectionalLight>().unwrap();
    let rotation = entity.get::<Rotation>().unwrap();

    renderer
        .scene_buffer
        .write(
            &[SceneUniform {
                sun_direction: rotation.forward().with_w(0.0),
                sun_color: vek::Rgba::<f32>::from(light.color),
                ..Default::default()
            }],
            0,
        )
        .unwrap();

    // Create the shared material resources
    let mut default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        white: &albedo_maps[&renderer.white],
        black: &albedo_maps[&renderer.black],
        normal: &normal_maps[&renderer.normal],
        sky_gradient: &renderer.sky_gradient,
        material_index: 0,
        draw_call_index: 0,
    };

    // Create some ECS filters to check if we should update the shadow map texture
    let f1 = ecs::modified::<ecs::Position>();
    let f2 = ecs::modified::<ecs::Rotation>();
    let f3 = ecs::modified::<ecs::Scale>();
    let f4 = f1 | f2 | f3;
    let mut update =
        scene.query_with::<&Renderer>(f4).into_iter().count() > 0;
    update |= scene
        .query_with::<&DirectionalLight>(f2)
        .into_iter()
        .count()
        > 0;

    if update {
        // Update the shadow map lightspace matrix
        let shadowmap = &mut *_shadowmap;
        shadowmap.update(**rotation);

        // Get the depth texture we will render to
        let depth = shadowmap.depth_tex.as_render_target().unwrap();

        // Create a new active shadowmap render pass
        let mut render_pass = shadowmap.render_pass.begin((), depth);

        // Bind the default shadowmap graphics pipeline
        let mut active =
            render_pass.bind_pipeline(&shadowmap.pipeline);

        // Bind the shadow map UBO that contains the matrices and parameters
        active.set_bind_group(0, |group| {
            group
                .set_uniform_buffer("shadow", &shadowmap.buffer)
                .unwrap();
        });

        // Render the shadows first (fuck you)
        for pipeline in pipelines.iter() {
            pipeline.prerender(world, &meshes, &mut active);
        }
        drop(active);
        drop(render_pass);
        drop(shadowmap);
    }
    drop(_shadowmap);

    // Begin the scene color render pass
    let color = renderer.color_texture.as_render_target().unwrap();
    let depth = renderer.depth_texture.as_render_target().unwrap();
    let mut render_pass = renderer.render_pass.begin(color, depth);

    // This will iterate over each material pipeline and draw the scene
    for pipeline in pipelines.iter() {
        pipeline.render(
            world,
            &meshes,
            &mut default,
            &mut render_pass,
        );
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
