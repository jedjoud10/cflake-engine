use assets::Assets;

use graphics::{ActivePipeline, Graphics, Texture, Window};

use utils::Time;
use world::{world::World, events::{Init, Update}, system::{Registries, pre_user}};

use crate::scene::{Compositor, PostProcessUniform, DeferredRenderer, Environment, ShadowMapping};

// Inserts the compositor render pass
fn init(world: &mut World, _: &Init) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let pp = Compositor::new(&graphics, &mut assets);
    drop(graphics);
    drop(assets);
    world.insert(PostProcessUniform::default());
    world.insert(pp);
}

// Displays the rendered scene texture to the actual window texture (post-processing pass)
fn update(world: &mut World, _: &Update) {
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let mut compositor = world.get_mut::<Compositor>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let environment = world.get::<Environment>().unwrap();
    let shadow = world.get::<ShadowMapping>().unwrap();
    let _time = world.get::<Time>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();

    // Write the post process settings to the buffer
    let value = compositor.post_process;
    compositor.post_process_buffer.write(&[value], 0).unwrap();

    // Get G-Buffer sampled textures
    let gbuffer_albedo_map = &renderer.gbuffer_albedo_texture;
    let gbuffer_normal_map = &renderer.gbuffer_normal_texture;
    let gbuffer_mask_map = &renderer.gbuffer_mask_texture;
    let depth_map = &renderer.depth_texture;

    let Some(dst) = window.as_render_target() else {
        return;
    };

    // Begin the render pass and bind the composite shader
    let mut encoder = graphics.acquire();
    let mut render_pass = compositor.lighting_render_pass.begin(&mut encoder, dst, ());
    let mut active = render_pass.bind_pipeline(&compositor.lighting_pipeline);

    /*
    // Set the shared UBOs first (bind group 0)
    active
        .set_bind_group(0, |group| {
            group
                .set_uniform_buffer("camera", &renderer.camera_buffer, ..)
                .unwrap();
            group
                .set_uniform_buffer("window", &renderer.window_buffer, ..)
                .unwrap();
            group
                .set_uniform_buffer("scene", &renderer.scene_buffer, ..)
                .unwrap();
            group
                .set_uniform_buffer("post_processing", &compositor.post_process_buffer, ..)
                .unwrap();
            group
                .set_uniform_buffer("shadow_parameters", &shadow.parameter_buffer, ..)
                .unwrap();
            group
                .set_uniform_buffer("shadow_lightspace_matrices", &shadow.lightspace_buffer, ..)
                .unwrap();
            group
                .set_sampled_texture("shadow_map", &shadow.dynamic_depth_tex)
                .unwrap();
            group
                .set_sampler("shadow_map_sampler", shadow.static_depth_tex.sampler().unwrap())
                .unwrap();
            group
                .set_sampled_texture("environment_map", &environment.environment_map)
                .unwrap();
            group
                .set_sampler(
                    "environment_map_sampler",
                    environment.environment_map.sampler().unwrap(),
                )
                .unwrap();
            group
                .set_sampled_texture("ibl_diffuse_map", &environment.diffuse_ibl_map)
                .unwrap();
            group
                .set_sampler(
                    "ibl_diffuse_map_sampler",
                    environment.diffuse_ibl_map.sampler().unwrap(),
                )
                .unwrap();
        })
        .unwrap();

    // Set the maps that we will sample
    active
        .set_bind_group(1, |group| {
            group
                .set_sampled_texture("gbuffer_albedo_map", gbuffer_albedo_map)
                .unwrap();
            group
                .set_sampled_texture("gbuffer_normal_map", gbuffer_normal_map)
                .unwrap();
            group
                .set_sampled_texture("gbuffer_mask_map", gbuffer_mask_map)
                .unwrap();
            group.set_sampled_texture("depth_map", depth_map).unwrap();
        })
        .unwrap();
    */

    // Draw 6 vertices (2 tris)
    active.draw(0..6, 0..1).unwrap();
}

// The display plugin will be responsible for displaying the renderered scene textures to the scene
pub fn plugin(registries: &mut Registries) {
    registries.init
        .insert(init)
        .before(pre_user)
        .after(assets::init)
        .after(graphics::init);
    registries.update
        .insert(update)
        .after(super::rendering::update)
        .after(graphics::acquire)
        .before(graphics::present);
}
