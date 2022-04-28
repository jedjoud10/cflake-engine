use cflake_engine::{
    assets, defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType, texture::{TextureParams, Texture2D, TextureFilter},
    },
    vek, World,
};
// An example with a test mesh
fn main() {
    cflake_engine::start("cflake-examples/mesh", init)
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
        let light = Light(LightType::new_directional(2.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-45f32.to_radians())).unwrap();
    });

    // Create a sphere
    let sphere = world.pipeline.defaults().sphere.clone();
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::from(sphere)).unwrap();
        linker.insert(Transform::at_y(0.5)).unwrap();
    });

    // Create a floor
    let plane = world.pipeline.defaults().plane.clone();
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::from(plane)).unwrap();
        linker.insert(Transform::default().scaled_by(vek::Vec3::new(10.0, 1.0, 10.0))).unwrap();
    });
}
