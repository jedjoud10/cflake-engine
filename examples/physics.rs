use cflake_engine::{
    assets::{self, assetc},
    defaults::components::{self, Camera, Collider, ColliderType, Renderer, RigidBody, RigidBodyType, Transform},
    ecs::entity::{ComponentLinkingGroup, Entity},
    math::shapes::{Cuboid, ShapeType},
    rendering::{
        basics::{
            lights::{LightSource, LightSourceType},
            renderer::RendererFlags,
        },
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
    group.link(Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link_default::<Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.add_entity(entity, group).unwrap();
    let pipeline = world.pipeline.read();
    // Create the directional light source
    let light = LightSource::new(LightSourceType::Directional {
        quat: veclib::Quaternion::<f32>::from_x_angle(-90f32.to_radians()),
    })
    .with_strength(1.0);
    pipec::construct(&pipeline, light).unwrap();

    // Create a flat surface
    let mut group = ComponentLinkingGroup::default();
    group.link(Transform::default().with_scale(veclib::Vector3::new(50.0, 2.0, 50.0))).unwrap();
    let renderer = Renderer::new(RendererFlags::DEFAULT).with_mesh(pipeline.defaults.as_ref().unwrap().plane);
    group.link(renderer).unwrap();
    // Add the rigidbody
    group.link(RigidBody::new(RigidBodyType::Static)).unwrap();
    // Add the collider
    group
        .link(Collider::new(ColliderType::Shape(ShapeType::Cuboid(Cuboid {
            center: veclib::Vector3::ZERO,
            size: veclib::Vector3::new(50.0, 2.0, 50.0),
        }))))
        .unwrap();
    let entity = Entity::default();
    world.ecs.add_entity(entity, group).unwrap();

    // Create a cube
    let mut group = ComponentLinkingGroup::default();
    group.link(Transform::default().with_position(veclib::Vector3::Y * 20.0)).unwrap();
    let renderer = Renderer::new(RendererFlags::DEFAULT).with_mesh(pipeline.defaults.as_ref().unwrap().cube);
    group.link(renderer).unwrap();
    // Add the rigidbody
    group.link(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
    // Add the collider
    group
        .link(Collider::new(ColliderType::Shape(ShapeType::Cuboid(Cuboid {
            center: veclib::Vector3::ZERO,
            size: veclib::Vector3::new(1.0, 1.0, 1.0),
        }))))
        .unwrap();
    let entity = Entity::default();
    world.ecs.add_entity(entity, group).unwrap();
}
