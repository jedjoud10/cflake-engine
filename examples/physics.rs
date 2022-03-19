use cflake_engine::{
    defaults::components::{Camera, Collider, ColliderGeometry, ColliderMaterial, Light, Renderer, RigidBody, RigidBodyType, Transform},
    ecs::entity::ComponentLinkingGroup,
    rendering::basics::lights::{LightParameters, LightType},
    vek, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("cflake-examples", "physics", init, cflake_engine::defaults::load_debugging_systems)
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
    // Add the rigidbody
    group.link(RigidBody::new(RigidBodyType::Static)).unwrap();
    // Add the collider
    group
        .link(Collider::new(ColliderGeometry::cuboid(vek::Vec3::new(50.0, 1.0, 50.0)), ColliderMaterial::new(10.0, 0.0)))
        .unwrap();
    world.ecs.add(group).unwrap();
    for y in 0..20 {
        for x in 0..5 {
            for z in 0..5 {
                // Create a cube
                let mut group = ComponentLinkingGroup::default();
                group
                    .link(Transform {
                        position: vek::Vec3::new(x as f32 * 0.3, y as f32 * 2.0 + 20.0, z as f32 * 0.3),
                        scale: vek::Vec3::new(1.0, 1.0, 1.0),
                        ..Default::default()
                    })
                    .unwrap();
                let renderer = Renderer {
                    mesh: world.pipeline.defaults().cube.clone(),
                    ..Default::default()
                };
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
                // Add the collider
                group
                    .link(Collider::new(ColliderGeometry::cuboid(vek::Vec3::one()), ColliderMaterial::new(10.0, 0.0)))
                    .unwrap();
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
                        position: vek::Vec3::new(x as f32 * 0.3, y as f32 * 2.0 + 50.0, z as f32 * 0.3),
                        scale: vek::Vec3::new(1.0, 1.0, 1.0),
                        ..Default::default()
                    })
                    .unwrap();
                let renderer = Renderer {
                    mesh: world.pipeline.defaults().sphere.clone(),
                    ..Default::default()
                };
                group.link(renderer).unwrap();
                // Add the rigidbody
                group.link(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
                // Add the collider
                group.link(Collider::new(ColliderGeometry::sphere(0.5), ColliderMaterial::new(10.0, 0.0))).unwrap();
                world.ecs.add(group).unwrap();
            }
        }
    }
}
