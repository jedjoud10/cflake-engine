use cflake_engine::{
    defaults::components,
    ecs::entity::{ComponentLinkingGroup, Entity},
    rendering::{
        basics::lights::{LightSource, LightSourceType},
        pipeline::pipec,
    },
    veclib, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-camera", init)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(components::Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link_default::<components::Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.add_entity(entity, group).unwrap();
    let pipeline = world.pipeline.read();
    // Create the directional light source
    let light = LightSource::new(LightSourceType::Directional {
        quat: veclib::Quaternion::<f32>::from_x_angle(-90f32.to_radians()),
    })
    .with_strength(1.0);
    pipec::construct(&pipeline, light).unwrap();
}
