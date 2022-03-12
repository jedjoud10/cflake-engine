use cflake_engine::{
    defaults::components::{Camera, Collider, Light, Renderer, RigidBody, RigidBodyType, Transform, ColliderGeometry, ColliderMaterial},
    ecs::entity::ComponentLinkingGroup,
    rendering::basics::lights::{LightParameters, LightType},
    veclib, World, math::shapes::ShapeType,
};
// A game with a test camera
fn main() {
    cflake_engine::start("DevJed", "cflake-engine-example-physics", init, cflake_engine::defaults::systems::flycam_system::system)
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
        rotation: veclib::Quaternion::<f32>::from_x_angle(-30f32.to_radians()),
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
            scale: veclib::vec3(50.0, 1.0, 50.0),
            ..Default::default()
        })
        .unwrap();
    let renderer = Renderer {
        mesh: world.pipeline.defaults().cube.clone(),
        ..Default::default()
    };
    group.link(renderer).unwrap();
    // Add the rigidbody
    group.link(RigidBody::new(RigidBodyType::Static, veclib::Vector3::ZERO)).unwrap();
    // Add the collider
    group.link(Collider::new(ColliderGeometry::cuboid(veclib::Vector3::new(50.0, 1.0, 50.0)), ColliderMaterial::new(10.0, 0.0))).unwrap();
    world.ecs.add(group).unwrap();
    for y in 0..20 {
        for x in 0..5 {
            for z in 0..5 {
                // Create a cube
                let mut group = ComponentLinkingGroup::default();
                group
                    .link(Transform {
                        position: veclib::vec3(x as f32 * 0.3, y as f32 * 2.0 + 20.0, z as f32 * 0.3),
                        scale: veclib::vec3(1.0, 1.0, 1.0),
                        ..Default::default()
                    })
                    .unwrap();
                let renderer = Renderer {
                    mesh: world.pipeline.defaults().cube.clone(),
                    ..Default::default()
                };
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic, veclib::Vector3::ZERO)).unwrap();
                // Add the collider
                group.link(Collider::new(ColliderGeometry::cuboid(veclib::Vector3::ONE), ColliderMaterial::new(10.0, 0.0))).unwrap();
                world.ecs.add(group).unwrap();
            }
        }
    }
    for y in 0..20 {
        for x in 0..5 {
            for z in 0..5 {
                // Create a sphere
                let mut group = ComponentLinkingGroup::default();
                group
                    .link(Transform {
                        position: veclib::vec3(x as f32 * 0.3, y as f32 * 2.0 + 50.0, z as f32 * 0.3),
                        scale: veclib::vec3(1.0, 1.0, 1.0),
                        ..Default::default()
                    })
                    .unwrap();
                let renderer = Renderer {
                    mesh: world.pipeline.defaults().sphere.clone(),
                    ..Default::default()
                };
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic, veclib::Vector3::ZERO)).unwrap();
                // Add the collider
                group.link(Collider::new(ColliderGeometry::sphere(0.5), ColliderMaterial::new(10.0, 0.0))).unwrap();
                world.ecs.add(group).unwrap();
            }
        }
    }
}
