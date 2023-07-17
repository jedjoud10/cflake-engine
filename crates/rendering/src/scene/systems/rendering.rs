use crate::{
    create_gbuffer_texture, AlbedoMap, AttributeBuffer, Camera, DefaultMaterialResources,
    DeferredRenderer, DirectionalLight, DynPipeline, Environment, Indirect, IndirectMesh, MaskMap,
    Mesh, MultiDrawIndirectCountMesh, MultiDrawIndirectMesh, NormalMap, PassStats, PbrMaterial,
    Pipelines, SceneUniform, ShadowMapping, TimingUniform, WindowUniform,
};
use assets::Assets;

use ecs::Scene;
use graphics::{
    Depth, DrawCountIndirectBuffer, DrawIndexedIndirectBuffer, GpuPod, Graphics, Normalized,
    Texture, TriangleBuffer, Window, RGBA,
};

use utils::{Storage, Time};
use world::{post_user, user, System, WindowEvent, World};

// Add the scene resources and setup for rendering
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let assets = world.get_mut::<Assets>().unwrap();
    let mut albedo_maps = Storage::<AlbedoMap>::default();
    let mut normal_maps = Storage::<NormalMap>::default();
    let mut mask_maps = Storage::<MaskMap>::default();
    let mut meshes = Storage::<Mesh>::default();

    // Create the scene renderer, pipeline manager
    let renderer = DeferredRenderer::new(
        &graphics,
        &assets,
        window.size(),
        &mut meshes,
        &mut albedo_maps,
        &mut normal_maps,
        &mut mask_maps,
    );

    // Pre-initialize the pipeline with the material types
    let mut pipelines = Pipelines::new();
    pipelines
        .register::<PbrMaterial>(&graphics, &assets)
        .unwrap();

    // Create a nice shadow map
    let shadowmap = ShadowMapping::new(4096, [0.05, 0.15, 0.75, 1.0], 1000.0, &graphics);

    // Drop fetched resources
    drop(graphics);
    drop(window);
    drop(assets);

    // Add composites and basic storages
    world.insert(renderer);
    world.insert(pipelines);
    world.insert(shadowmap);

    // Add mesh storages
    world.insert(meshes);
    world.insert(Storage::<IndirectMesh>::default());
    world.insert(Storage::<MultiDrawIndirectMesh>::default());
    world.insert(Storage::<MultiDrawIndirectCountMesh>::default());

    // Add common indirect attributes
    world.insert(Storage::<AttributeBuffer<crate::attributes::Position>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::Normal>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::Tangent>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::TexCoord>>::default());
    world.insert(Storage::<TriangleBuffer<u32>>::default());

    // Add draw indexed indirect buffers
    world.insert(Storage::<DrawIndexedIndirectBuffer>::default());
    world.insert(Storage::<DrawCountIndirectBuffer>::default());

    // Add the storages that contain the materials and their resources
    world.insert(Storage::<PbrMaterial>::default());
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

            // Resize the G-Buffer textures and Depth texture
            let size = vek::Extent2::new(size.width, size.height);
            let mut renderer = world.get_mut::<DeferredRenderer>().unwrap();

            // Skip resizing if it's the same size as before
            if renderer.window_size == size {
                return;
            }

            let _graphics = world.get::<Graphics>().unwrap();
            let graphics = &*_graphics;

            // Create new textures with the new size
            renderer.window_size = size;
            let new_gbuffer_albedo_texture =
                create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, size);
            let new_gbuffer_normal_texture =
                create_gbuffer_texture::<RGBA<Normalized<i8>>>(graphics, size);
            let new_gbuffer_mask_texture =
                create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, size);
            let new_depth_texture = create_gbuffer_texture::<Depth<f32>>(graphics, size);

            // Set the new as the old textures
            renderer.gbuffer_albedo_texture = new_gbuffer_albedo_texture;
            renderer.gbuffer_normal_texture = new_gbuffer_normal_texture;
            renderer.gbuffer_mask_texture = new_gbuffer_mask_texture;
            renderer.depth_texture = new_depth_texture;

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
    let mut renderer = world.get_mut::<DeferredRenderer>().unwrap();
    let mut _shadowmap = world.get_mut::<ShadowMapping>().unwrap();
    let renderer = &mut *renderer;
    let scene = world.get::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let time = world.get::<Time>().unwrap();
    let _graphics = world.get::<Graphics>().unwrap();
    let environment = world.get::<Environment>().unwrap();

    // Reset the stats
    renderer.deferred_pass_stats = Default::default();
    renderer.shadow_pass_stats = Default::default();

    // Needed for direct rendering
    let meshes = world.get::<Storage<Mesh>>().unwrap();

    // Needed for indirect rendering
    let indirect_meshes = world.get::<Storage<Mesh<Indirect>>>().unwrap();
    let indirect_position_attribute = world
        .get::<Storage<AttributeBuffer<crate::attributes::Position>>>()
        .unwrap();
    let indirect_normal_attribute = world
        .get::<Storage<AttributeBuffer<crate::attributes::Normal>>>()
        .unwrap();
    let indirect_tangents_attribute = world
        .get::<Storage<AttributeBuffer<crate::attributes::Tangent>>>()
        .unwrap();
    let indirect_tex_coords_attribute = world
        .get::<Storage<AttributeBuffer<crate::attributes::TexCoord>>>()
        .unwrap();
    let indexed_indirect_buffers = world.get::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let indirect_triangles = world.get::<Storage<TriangleBuffer<u32>>>().unwrap();

    // Needed for multi draw indirect rendering
    let multi_draw_indirect_meshes = world.get::<Storage<MultiDrawIndirectMesh>>().unwrap();
    let multi_draw_indirect_count_meshes =
        world.get::<Storage<MultiDrawIndirectCountMesh>>().unwrap();
    let draw_count_indirect_buffer = world.get::<Storage<DrawCountIndirectBuffer>>().unwrap();

    let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
    let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
    let mask_maps = world.get::<Storage<MaskMap>>().unwrap();
    let pipelines = pipelines.extract_pipelines();

    // Skip if we don't have a camera to draw with
    let Some(camera) = renderer.main_camera else {
        return;
    };

    // Skip if we don't have a light to draw with
    let Some(directional_light)  = renderer.main_directional_light else {
        return;
    };

    // Get the directioanl light and rotation of the light
    let directional_light = scene.entry(directional_light).unwrap();
    let (&directional_light, &directional_light_rotation) = directional_light
        .as_query::<(&DirectionalLight, &coords::Rotation)>()
        .unwrap();

    // Update the scene uniform using the appropriate values
    renderer
        .scene_buffer
        .write(
            &[SceneUniform {
                sun_direction: directional_light_rotation.forward().with_w(0.0),
                sun_color: vek::Rgba::<f32>::from(
                    directional_light.color.map(|x| x as f32 / 255.0) * directional_light.intensity,
                ),
                ..Default::default()
            }],
            0,
        )
        .unwrap();

    // Store the new timing info
    renderer
        .timing_buffer
        .write(
            &[TimingUniform {
                frame_count: time.frame_count().try_into().unwrap(),
                delta_time: time.delta().as_secs_f32(),
                time_since_startup: time.startup().elapsed().as_secs_f32(),
            }],
            0,
        )
        .unwrap();

    // Get the camera and it's values
    let camera = scene.entry(camera).unwrap();
    let (&camera, &camera_position, &camera_rotation) = camera
        .as_query::<(&Camera, &coords::Position, &coords::Rotation)>()
        .unwrap();
    let camera_view = camera.view_matrix(&camera_position, &camera_rotation);
    let camera_projection = camera.projection_matrix();
    let camera_frustum = math::Frustum::<f32>::from_camera_matrices(camera_projection, camera_view);

    // Create the shared material resources
    let mut default = DefaultMaterialResources {
        camera_buffer: &renderer.camera_buffer,
        timing_buffer: &renderer.timing_buffer,
        scene_buffer: &renderer.scene_buffer,
        camera,
        camera_frustum,
        camera_position,
        camera_rotation,
        directional_light,
        directional_light_rotation,
        white: &albedo_maps[&renderer.white],
        black: &albedo_maps[&renderer.black],
        normal: &normal_maps[&renderer.normal],
        mask: &mask_maps[&renderer.mask],
        meshes: &meshes,
        indirect_meshes: &indirect_meshes,
        multi_draw_indirect_meshes: &multi_draw_indirect_meshes,
        multi_draw_indirect_count_meshes: &multi_draw_indirect_count_meshes,
        draw_count_indirect_buffer: &draw_count_indirect_buffer,
        indirect_positions: &indirect_position_attribute,
        indirect_normals: &indirect_normal_attribute,
        indirect_tangents: &indirect_tangents_attribute,
        indirect_tex_coords: &indirect_tex_coords_attribute,
        indirect_triangles: &indirect_triangles,
        draw_indexed_indirect_buffers: &indexed_indirect_buffers,
        lightspace: None,
        environment_map: &environment.environment_map,
    };
    drop(scene);

    // Render into a single cascade of the shadow map
    fn render_shadows_pipelines(
        shadowmap: &mut ShadowMapping,
        default: &mut DefaultMaterialResources,
        directional_light_rotation: coords::Rotation,
        camera_position: coords::Position,
        stats: &mut PassStats,
        index: usize,
        pipelines: &Vec<std::rc::Rc<dyn DynPipeline>>,
        world: &World,
    ) {
        default.lightspace =
            Some(shadowmap.update(*directional_light_rotation, *camera_position, index));

        let mut view = shadowmap.depth_tex.view_mut(1 + index).unwrap();
        let target = view.as_render_target().unwrap();

        // Create a new active shadowmap render pass
        let mut render_pass = shadowmap.render_pass.begin((), target);

        // Render the shadows first (fuck you)
        for stored in pipelines.iter() {
            stored.render_shadow(world, default, stats, &mut render_pass);
        }

        drop(render_pass);
    }

    // Render to the shadow map cascades
    if _shadowmap.distance > 0.0 {
        for i in 0..4 {
            render_shadows_pipelines(
                &mut _shadowmap,
                &mut default,
                directional_light_rotation,
                camera_position,
                &mut renderer.shadow_pass_stats,
                i,
                &pipelines,
                world,
            );
        }

        //graphics.submit(false);
        drop(_shadowmap);
    }

    // Begin the scene color render pass
    let gbuffer_albedo = renderer.gbuffer_albedo_texture.as_render_target().unwrap();
    let gbuffer_normal = renderer.gbuffer_normal_texture.as_render_target().unwrap();
    let gbuffer_mask = renderer.gbuffer_mask_texture.as_render_target().unwrap();
    let gbuffer = (gbuffer_albedo, gbuffer_normal, gbuffer_mask);
    let depth = renderer.depth_texture.as_render_target().unwrap();
    let mut render_pass = renderer.deferred_render_pass.begin(gbuffer, depth);

    // This will iterate over each material pipeline and draw the scene
    default.lightspace = None;
    for stored in pipelines.iter() {
        stored.render(
            world,
            &mut default,
            &mut renderer.deferred_pass_stats,
            &mut render_pass,
        );
    }

    drop(render_pass);
    //dbg!(i.elapsed());
    //graphics.submit(false);
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
        .after(post_user)
        .after(graphics::acquire)
        .before(graphics::present);
    system
        .insert_window(event)
        .after(graphics::common)
        .before(user);
}
