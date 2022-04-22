use cflake_engine::{
    assets,
    defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType,
        material::{MaterialBuilder, PbrMaterialBuilder},
        mesh::Mesh,
        texture::{Texture2D, TextureParams},
    },
    vek, World,
};
// An example with a test mesh
fn main() {
    cflake_engine::start("cflake-examples/textured-mesh", init)
}
// Init the simple camera and simple mesh
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");

    defaults::systems::flycam_system::system(world);

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 0.2, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });

    // Simple material with textures
    let mesh = assets::load::<Mesh>("user/meshes/untitled.obj").unwrap();
    let mesh = world.pipeline.insert(mesh);
    let diff = assets::load_with::<Texture2D>("user/textures/wooden_crate_01_diff_1k.jpg", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
    let norm = assets::load_with::<Texture2D>("user/textures/wooden_crate_01_nor_gl_1k.jpg", TextureParams::NORMAL_MAP_LOAD).unwrap();
    let diff = world.pipeline.insert(diff);
    let norm = world.pipeline.insert(norm);
    let material = PbrMaterialBuilder::default()
        .diffuse(diff)
        .normal(norm)
        .bumpiness(2.0)
        .scale(vek::Vec2::one() * 1.0)
        .build(&mut world.pipeline);

    for x in 0..20 {
        // Create an entity
        world.ecs.insert(|_, linker| {
            linker.insert(Renderer::new(mesh.clone(), material.clone())).unwrap();
            linker.insert(Transform::at_x(x as f32 * 2.0)).unwrap();
        });
    }
}
