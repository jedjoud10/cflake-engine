use crate::{
    AlbedoMap, Sky, Camera, CameraUniform,
    DefaultMaterialResources, ForwardRenderer,
    SceneRenderPass, Mesh, NormalMap, Pipelines, Renderer, Basic, PostProcess, WindowUniform, Compositor,
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


// Inserts the compositor render pass and the composites
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let pp = Compositor::new(&graphics, &mut assets);
    drop(graphics);
    drop(assets);
    world.insert(PostProcess::default());
    world.insert(pp);
}

// Displays the rendered scene texture to the actual window texture (post-processing pass)
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let renderer = world.get::<ForwardRenderer>().unwrap();
    let compositor = world.get::<Compositor>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();

    // Get textures, pipelines, and encoder
    let src = &renderer.color_texture;
    let dst = window.as_render_target().unwrap();
    let mut encoder = graphics.acquire();

    // Begin the render pass
    let mut render_pass = 
        compositor.render_pass.begin(&mut encoder, dst, ()).unwrap();

    // Bind the graphics pipeline
    let mut active = render_pass.bind_pipeline(&compositor.pipeline);

    // Set the required shader uniforms
    active.set_bind_group(0, |group| {
        group.set_texture("color_map", src).unwrap();
        group.set_sampler("color_map_sampler", src.sampler()).unwrap();
        group.set_buffer("window", &renderer.window_buffer).unwrap();
    });

    // Draw 6 vertices (2 tris)
    active.draw(0..6, 0..1);

    // Submit the encoder at the end
    drop(render_pass);
    graphics.submit([encoder]);
}

// The display system will be responsible for displaying the renderered scene textures to the scene
pub fn system(system: &mut System) {
    system.insert_init(init)
        .before(user)
        .after(graphics::common);
    system.insert_update(update)
        .after(super::rendering::system)
        .after(graphics::acquire)
        .before(graphics::present);
}
