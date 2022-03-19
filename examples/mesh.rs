use cflake_engine::{
    defaults::components::{Camera, Light, Renderer, Transform},
    ecs::entity::ComponentLinkingGroup,
    rendering::basics::lights::{LightParameters, LightType},
    vek, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("cflake-examples", "mesh", init, cflake_engine::defaults::load_debugging_systems)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link(Transform::default()).unwrap();
    world.ecs.add(group).unwrap();

    // Create the directional light source
    let light = Light {
        light: LightType::Directional {
            params: LightParameters::default(),
        },
    };
    let light_transform = Transform {
        rotation: vek::Quaternion::<f32>::rotation_x(-30f32.to_radians()),
        ..Default::default()
    };
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(group).unwrap();

    // Create a flat surface
    let mut group = ComponentLinkingGroup::default();
    group
        .link(Transform {
            scale: vek::Vec3::new(50.0, 1.0, 50.0),
            ..Default::default()
        })
        .unwrap();
    let renderer = Renderer {
        mesh: world.pipeline.defaults().cube.clone(),
        ..Default::default()
    };
    group.link(renderer).unwrap();
    world.ecs.add(group).unwrap();
}
