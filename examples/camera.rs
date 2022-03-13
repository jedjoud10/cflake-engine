use cflake_engine::{
    assets,
    defaults::components::{self, Camera, Transform},
    ecs::entity::ComponentLinkingGroup,
    rendering::basics::lights::{LightParameters, LightType::Directional},
    vek, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-camera", init, cflake_engine::defaults::systems::flycam_system::system)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link(Transform::default()).unwrap();
    world.ecs.add(group).unwrap();
    // Create the directional light source
    let light = components::Light {
        light: Directional {
            params: LightParameters::default(),
        },
    };
    let light_transform = Transform {
        rotation: vek::Quaternion::<f32>::rotation_x(-90f32.to_radians()),
        ..Default::default()
    };
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(group).unwrap();
}
