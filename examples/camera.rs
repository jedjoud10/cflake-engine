use cflake_engine::{
    assets,
    defaults::components::{self, Camera, Light, Transform},
    rendering::basics::lights::{
        LightParameters,
        LightType::{self, Directional},
    },
    vek, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("cflake-examples/camera", init)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");
    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::default()).unwrap();
    });
    /*
    let light = components::Light {
        light: Directional {
            params: LightParameters::default(),
        },
    };
    let light_transform = Transform {
        rotation: vek::Quaternion::<f32>::rotation_x(-90f32.to_radians()),
        ..Default::default()
    };
    */
    // And add it to the world as an entity
}
