use std::{mem::size_of, num::NonZeroU8};

use crate::{
    AlbedoMap, AttributeBuffer, BasicMaterial, Camera, DefaultMaterialResources, DirectionalLight,
    ForwardRenderer, Indirect, MaskMap, Mesh, NormalMap, PhysicallyBasedMaterial, Pipelines,
    Renderer, SceneUniform, ShadowMapping, SkyMaterial, WindowUniform, MultiDrawIndirectMesh, IndirectMesh, WireframeMaterial,
};
use assets::Assets;

use ecs::Scene;
use graphics::{
    ActivePipeline, DrawIndexedIndirectBuffer, GpuPod, Graphics, ModuleVisibility, Texture,
    TriangleBuffer, Window,
};

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
    pipelines
        .register::<BasicMaterial>(&graphics, &assets)
        .unwrap();
    pipelines
        .register::<SkyMaterial>(&graphics, &assets)
        .unwrap();
    pipelines
        .register::<PhysicallyBasedMaterial>(&graphics, &assets)
        .unwrap();

    // Create a nice shadow map
    let shadowmap = ShadowMapping::new(
        2000f32,
        2048,
        [0.003, 0.01, 0.02, 0.05],
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
    world.insert(Storage::<IndirectMesh>::default());
    world.insert(Storage::<MultiDrawIndirectMesh>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::Position>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::Normal>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::Tangent>>::default());
    world.insert(Storage::<AttributeBuffer<crate::attributes::TexCoord>>::default());
    world.insert(Storage::<TriangleBuffer<u32>>::default());
    world.insert(Storage::<DrawIndexedIndirectBuffer>::default());

    // Add the storages that contain the materials and their resources
    world.insert(Storage::<BasicMaterial>::default());
    world.insert(Storage::<SkyMaterial>::default());
    world.insert(Storage::<PhysicallyBasedMaterial>::default());
    world.insert(Storage::<WireframeMaterial>::default());
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
            let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();

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
    let time = world.get::<Time>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();

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
                sun_color: vek::Rgba::<f32>::from(directional_light.color),
                ..Default::default()
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
        meshes: &meshes,
        indirect_meshes: &indirect_meshes,
        multi_draw_indirect_meshes: &multi_draw_indirect_meshes,
        indirect_positions: &indirect_position_attribute,
        indirect_normals: &indirect_normal_attribute,
        indirect_tangents: &indirect_tangents_attribute,
        indirect_tex_coords: &indirect_tex_coords_attribute,
        indirect_triangles: &indirect_triangles,
        draw_indexed_indirect_buffers: &indexed_indirect_buffers,
    };
    drop(scene);

    /*
    // Update the shadow map lightspace matrix
    let shadowmap = &mut *_shadowmap;
    let index = (time.frame_count() as u32) % 4;
    let lightspace = shadowmap.update(
        *directional_light_rotation,
        camera_view,
        camera_projection,
        *camera_position,
        camera.near,
        camera.far,
        index as usize
    );
    let mips = shadowmap.depth_tex.mips_mut();
    let mut level = mips.level_mut(0).unwrap();

    // Use layer as render target
    let target = level.layer_as_render_target(index).unwrap();

    // Create a new active shadowmap render pass
    let mut render_pass = shadowmap.render_pass.begin((), target);

    // Get the default shadowmap render pipeline
    let default_shadow_pipeline = &shadowmap.pipeline;

    // Render the shadows first (fuck you)
    for stored in pipelines.iter() {
        stored.render_shadows(world, &mut default, &mut render_pass, default_shadow_pipeline, lightspace);
    }

    // Send the command encoder
    drop(default_shadow_pipeline);
    drop(render_pass);
    drop(level);
    drop(mips);
    drop(shadowmap);
    */

    // Drop resources
    drop(_shadowmap);

    // Begin the scene color render pass
    let color = renderer.color_texture.as_render_target().unwrap();
    let depth = renderer.depth_texture.as_render_target().unwrap();
    let mut render_pass = renderer.render_pass.begin(color, depth);

    // This will iterate over each material pipeline and draw the scene
    for stored in pipelines.iter() {
        stored.render(world, &mut default, &mut render_pass);
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
