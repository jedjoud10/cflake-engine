use cflake_engine::{
    assets,
    defaults::components,
    ecs::entity::{ComponentLinkingGroup, Entity},
    rendering::basics::lights::LightSourceType,
    veclib, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-camera", init)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(components::Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link_default::<components::Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.entities.add(entity).unwrap();
    world.ecs.components.link(_id, &mut world.ecs.entities, &mut world.ecs.systems, group).unwrap();
    // Create the directional light source
    let light = components::Light::default();
    let light_transform = components::Transform::default().with_rotation(veclib::Quaternion::<f32>::from_x_angle(-90f32.to_radians()));
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(Entity::default(), group).unwrap();
}
