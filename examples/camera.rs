use cflake_engine::{
    core::World,
    defaults::{components, globals},
    ecs::entity::{ComponentLinkingGroup, Entity, EntityID},
    rendering::{
        basics::lights::{LightSource, LightSourceType},
        pipeline::pipec,
    },
    veclib,
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
    group
        .link(components::Camera::new(90.0, 2.0, 9000.0))
        .unwrap();
    group.link_default::<components::Transform>().unwrap();
    let entity = Entity::default();
    let id = EntityID::new(&mut world.ecs);
    world.ecs.add_entity(entity, id, group).unwrap();
    let pipeline = world.pipeline.read();
    // Create the directional light source
    let light = LightSource::new(LightSourceType::Directional {
        quat: veclib::Quaternion::<f32>::from_x_angle(-90f32.to_radians()),
    })
    .with_strength(1.0);
    let mut world_global = world
        .globals
        .get_global_mut::<globals::GlobalWorldData>()
        .unwrap();
    world_global.sun_quat = veclib::Quaternion::<f32>::from_axis_angle(veclib::Vector3::X, 80.0);
    pipec::construct(&pipeline, light).unwrap();
}
