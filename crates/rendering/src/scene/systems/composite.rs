use crate::{Compositor, DeferredRenderer, PostProcessUniform};
use assets::Assets;

use graphics::{ActivePipeline, Graphics, Texture, Window};

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
    let _graphics = world.get::<Graphics>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let mut compositor = world.get_mut::<Compositor>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();

    // Write the post process settings to the buffer
    let value = compositor.post_process;
    compositor.post_process_buffer.write(&[value], 0).unwrap();

    // Get textures, pipelines, and encoder
    let src = &renderer.color_texture;
    let depth = &renderer.depth_texture;

    let Ok(dst) = window.as_render_target() else {
        return;
    };

    // Begin the render pass and bind the composite shader
    let mut render_pass = compositor.render_pass.begin(dst, ());
    let mut active = render_pass.bind_pipeline(&compositor.pipeline);

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
                .set_uniform_buffer("post_processing", &compositor.post_process_buffer, ..)
                .unwrap()
        })
        .unwrap();

    // Set the maps that we will sample
    active
        .set_bind_group(1, |group| {
            group.set_sampled_texture("color_map", src).unwrap();
            group.set_sampled_texture("depth_map", depth).unwrap();
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
