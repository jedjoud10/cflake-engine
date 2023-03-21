use crate::{
    AlbedoMap, Basic, DefaultMaterialResources, DirectionalLight,
    ForwardRenderer, MaskMap, Mesh, NormalMap, PhysicallyBased,
    Pipelines, Renderer, SceneUniform, ShadowMapping, Sky,
    WindowUniform, Camera,
};
use assets::Assets;

use ecs::{Rotation, Scene};
use graphics::{Graphics, Texture, Window, DrawIndexedIndirectBuffer};

use log::LevelFilter;
use utils::{Storage, Time};
use world::{user, System, WindowEvent, World};

// Add the scene resources and setup for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut albedo_maps = Storage::<AlbedoMap>::default();
    let mut normal_maps = Storage::<NormalMap>::default();
    let mut mask_maps = Storage::<MaskMap>::default();

    // Create the scene renderer, pipeline manager
    let renderer = ForwardRenderer::new(
        &graphics,
        &assets,
        window.size(),
        &mut albedo_maps,
        &mut normal_maps,
        &mut mask_maps,
    );

    // Pre-initialize the pipeline with the material types
    let mut pipelines = Pipelines::new();
    pipelines.register::<Basic>(&graphics, &assets).unwrap();
    pipelines.register::<Sky>(&graphics, &assets).unwrap();
    pipelines.register::<PhysicallyBased>(&graphics, &assets).unwrap();

    // Create a nice shadow map
    let shadowmap = ShadowMapping::new(
        20f32,
        100f32,
        1024,
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
    world.insert(Storage::<PhysicallyBased>::default());
    world.insert(Storage::<DrawIndexedIndirectBuffer>::default());
    world.insert(albedo_maps);
    world.insert(normal_maps);
    world.insert(mask_maps);
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
    // Fetch the resources that we will use for rendering the scene
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
    let mut _shadowmap = world.get_mut::<ShadowMapping>().unwrap();
    let renderer = &mut *renderer;
    let scene = world.get::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let indirect = world.get::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
    let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
    let mask_maps = world.get::<Storage<MaskMap>>().unwrap();
    let pipelines = pipelines.extract_pipelines();

    // Skip if we don't have a camera to draw with
    let Some(camera) = renderer.main_camera else {
        //log::warn!("No active camera to draw with!");
        return;
    };

    // Skip if we don't have a light to draw with
    let Some(directional_light)  = renderer.main_directional_light else {
        //log::warn!("No directional light to draw with!");
        return;
    };

    // Get the directioanl light and rotation of the light
    let directional_light = scene.entry(directional_light).unwrap();
    let (&directional_light, &directional_light_rotation) = 
        directional_light.as_query::<(&DirectionalLight, &ecs::Rotation)>().unwrap();

    // Update the scene uniform using the appropriate values
    renderer
        .scene_buffer
        .write(
            &[SceneUniform {
                sun_direction: directional_light_rotation.forward().with_w(0.0),
                sun_color: vek::Rgba::<f32>::from(directional_light.color),
                ..Default::default()
            }],
            0,
        )
        .unwrap();

    // Get the camera and it's values
    let camera = scene.entry(camera).unwrap();
    let (&camera, &camera_position, &camera_rotation) =
        camera.as_query::<(&Camera, &ecs::Position, &ecs::Rotation)>().unwrap();
    let camera_frustum = camera.frustum(&camera_position, &camera_rotation);

    // Create the shared material resources
    let mut default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        white: &albedo_maps[&renderer.white],
        black: &albedo_maps[&renderer.black],
        normal: &normal_maps[&renderer.normal],
        mask: &mask_maps[&renderer.mask],
        material_index: 0,
        draw_call_index: 0,
        camera,
        camera_position,
        camera_rotation,
        camera_frustum,
        directional_light,
        directional_light_rotation,
    };

    // Create some ECS filters to check if we should update the shadow map texture
    let f1 = ecs::modified::<ecs::Position>();
    let f2 = ecs::modified::<ecs::Rotation>();
    let f3 = ecs::modified::<ecs::Scale>();
    let f4 = f1 | f2 | f3;
    let mut update =
        scene.query_with::<&Renderer>(f4).into_iter().filter(|r| r.visible).count() > 0;
    update |= scene
        .query_with::<&DirectionalLight>(f2)
        .into_iter()
        .count()
        > 0;
    if update {
        // Update the shadow map lightspace matrix
        let shadowmap = &mut *_shadowmap;
        shadowmap.update(*directional_light_rotation, vek::Vec3::zero());

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
        for stored in pipelines.iter() {
            stored.prerender(world, &meshes, &indirect,&mut active);
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
    drop(scene);
    for stored in pipelines.iter() {
        stored.render(world, &meshes, &indirect, &mut default, &mut render_pass);
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
