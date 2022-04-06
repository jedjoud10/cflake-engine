use cflake_engine::{
    assets,
    defaults,
    defaults::components::{Camera, Light, Transform},
    rendering::basics::lights::LightType,
    vek, World,
};
// An example with a test camera
fn main() {
    cflake_engine::start("cflake-examples/camera", init)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");

    defaults::systems::flycam_system::system(world);

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });
}
