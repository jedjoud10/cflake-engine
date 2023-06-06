use crate::{Compositor, DeferredRenderer, PostProcessUniform, ShadowMap, ShadowMapping};
use assets::Assets;

use graphics::{ActivePipeline, Graphics, Texture, Window};

use utils::Time;
use world::{user, System, World};

// Inserts the compositor render pass
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let pp = Compositor::new(&graphics, &mut assets);
    drop(graphics);
    drop(assets);
    world.insert(PostProcessUniform::default());
    world.insert(pp);
}

// Displays the rendered scene texture to the actual window texture (post-processing pass)
fn update(world: &mut World) {
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let mut compositor = world.get_mut::<Compositor>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let shadow = world.get::<ShadowMapping>().unwrap();
    let time = world.get::<Time>().unwrap();

    // Write the post process settings to the buffer
    let value = compositor.post_process;
    compositor.post_process_buffer.write(&[value], 0).unwrap();

    // Get G-Buffer sampled textures 
    let gbuffer_position_map = &renderer.gbuffer_position_texture;
    let gbuffer_albedo_map = &renderer.gbuffer_albedo_texture;
    let gbuffer_normal_map = &renderer.gbuffer_normal_texture;
    let gbuffer_mask_map = &renderer.gbuffer_mask_texture;
    let depth_map = &renderer.depth_texture;

    //let depth = &renderer.depth_texture;

    let Ok(dst) = window.as_render_target() else {
        return;
    };

    // Begin the render pass and bind the composite shader
    let mut render_pass = compositor.lighting_render_pass.begin(dst, ());
    let mut active = render_pass.bind_pipeline(&compositor.lighting_pipeline);

    let index = (((time.frame_count() as usize % 8) as f32) / 8.0).round() as usize;

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
                .set_sampled_texture("shadow_map", &shadow.depth_tex)
                .unwrap();
        })
        .unwrap();

    // Set the maps that we will sample
    active
        .set_bind_group(1, |group| {
            group.set_sampled_texture("gbuffer_position_map", gbuffer_position_map).unwrap();
            group.set_sampled_texture("gbuffer_albedo_map", gbuffer_albedo_map).unwrap();
            group.set_sampled_texture("gbuffer_normal_map", gbuffer_normal_map).unwrap();
            group.set_sampled_texture("gbuffer_mask_map", gbuffer_mask_map).unwrap();
            group.set_sampled_texture("depth_map", depth_map).unwrap();
        })
        .unwrap();

    // Draw 6 vertices (2 tris)
    active.draw(0..6, 0..1).unwrap();
}

// The display system will be responsible for displaying the renderered scene textures to the scene
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .before(user)
        .after(assets::system)
        .after(graphics::common);
    system
        .insert_update(update)
        .after(super::rendering::system)
        .after(graphics::acquire)
        .before(graphics::present);
}
