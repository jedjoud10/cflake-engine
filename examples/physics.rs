use cflake_engine::{
    defaults::components::{Camera, Collider, ColliderType, Renderer, RigidBody, RigidBodyType, Transform},
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
    cflake_engine::start("DevJed", "cflake-engine-example-physics", init)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link_default::<Transform>().unwrap();
    let entity = Entity::default();
    let _id = world.ecs.add(entity, group).unwrap();
    let pipeline = world.pipeline.read();
    // Create the directional light source
    let light = LightSource::new(LightSourceType::Directional {
        quat: veclib::Quaternion::<f32>::from_x_angle(-15f32.to_radians()),
    })
    .with_strength(1.0);
    pipec::construct(&pipeline, light).unwrap();

    // Create a flat surface
    let mut group = ComponentLinkingGroup::default();
    group.link(Transform::default().with_scale(veclib::Vector3::new(50.0, 1.0, 50.0))).unwrap();
    let renderer = Renderer::new(RendererFlags::DEFAULT).with_mesh(pipeline.defaults.as_ref().unwrap().cube);
    group.link(renderer).unwrap();
    // Add the rigidbody
    group.link(RigidBody::new(RigidBodyType::Static)).unwrap();
    // Add the collider
    group
        .link(Collider::cuboid(veclib::Vector3::new(50.0, 1.0, 50.0))).unwrap();
    let entity = Entity::default();
    world.ecs.add(entity, group).unwrap();
    for y in 0..5 {
        for x in 0..15 {
            for z in 0..5 {
                // Create a cube
                let mut group = ComponentLinkingGroup::default();
                group.link(Transform::default().with_position(veclib::vec3(x as f32 * 0.3, y as f32 * 2.0 + 20.0, z as f32 * 0.3)).with_scale(veclib::vec3(1.0, 1.0, 1.0))).unwrap();
                let renderer = Renderer::new(RendererFlags::DEFAULT).with_mesh(pipeline.defaults.as_ref().unwrap().cube);
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
                // Add the collider
                group
                .link(Collider::cuboid(veclib::Vector3::ONE))
                .unwrap();
                let entity = Entity::default();
                world.ecs.add(entity, group).unwrap();
            }
        }
    }
    for y in 0..5 {
        for x in 0..15 {
            for z in 0..5 {
                // Create a sphere
                let mut group = ComponentLinkingGroup::default();
                group.link(Transform::default().with_position(veclib::vec3(x as f32 * 0.3, y as f32 * 2.0 + 20.0, z as f32 * 0.3)).with_scale(veclib::vec3(1.0, 1.0, 1.0))).unwrap();
                let renderer = Renderer::new(RendererFlags::DEFAULT).with_mesh(pipeline.defaults.as_ref().unwrap().sphere);
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
                // Add the collider
                group
                .link(Collider::sphere(0.5).with_friction(0.05).with_restitution(1.3))
                .unwrap();
                let entity = Entity::default();
                world.ecs.add(entity, group).unwrap();
            }
        }
    }
}