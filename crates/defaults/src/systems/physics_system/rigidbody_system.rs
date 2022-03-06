use crate::systems::physics_system::{quat_to_rotation, vec3_to_translation};
use world::physics::rapier3d;
use world::physics::rapier3d::na::{Isometry, Point, Point3};
use world::physics::rapier3d::prelude::{MassProperties, RigidBodyBuilder, SharedShape};
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Pipeline;
use world::World;
use world::{ecs::event::EventKey, physics::rapier3d::prelude::ColliderBuilder};

// Convert a rendering mesh to it's SharedShape counterpart
fn get_mesh(scale_matrix: &veclib::Matrix4x4<f32>, mesh: &Mesh) -> SharedShape {
    // Convert vertices and indices
    let vertices = mesh
        .vertices
        .positions
        .iter()
        // Scale the points by the scale matrix
        .map(|vertex| scale_matrix.mul_point(vertex))
        .map(|vertex| Point3::new(vertex.x, vertex.y, vertex.z))
        .collect::<Vec<Point3<f32>>>();
    let indices = mesh.indices.chunks_exact(3).map(|slice| slice.try_into().unwrap()).collect::<Vec<[u32; 3]>>();
    dbg!(mesh.indices.len());
    dbg!(mesh.vertices.len());
    // Done
    SharedShape::trimesh(vertices, indices)
}

// Get the Rapier3D shared shape from a collider components
fn get_shared_shape(pipeline: &Pipeline, scale_matrix: &veclib::Matrix4x4<f32>, collider: &crate::components::Collider) -> SharedShape {
    match &collider._type {
        crate::components::ColliderType::Shape(shape) => match shape {
            world::math::shapes::ShapeType::Cuboid(cuboid) => SharedShape::cuboid(cuboid.size.x / 2.0, cuboid.size.y / 2.0, cuboid.size.z / 2.0),
            world::math::shapes::ShapeType::Sphere(sphere) => SharedShape::ball(sphere.radius),
        },
        crate::components::ColliderType::Mesh(mesh) => get_mesh(scale_matrix, pipeline.meshes.get(*mesh).unwrap()),
    }
}

// Whenever we add a rigidbody that has a collider attached to it, we must add them to the Rapier3D simulation
fn added_entities(world: &mut World, mut data: EventKey) {
    // Add each rigidbody and it's corresponding collider
    let query = data.as_query_mut().unwrap();
    let pipeline = world.pipeline.read();
    for (_, components) in query.iter_mut() {
        // Get the components
        let rigidbody = components.get::<crate::components::RigidBody>().unwrap();
        let collider = components.get::<crate::components::Collider>().unwrap();
        let transform = components.get::<crate::components::Transform>().unwrap();

        // Transform to Rapier3D collider and rigibody
        let r_rigibody = RigidBodyBuilder::new(rigidbody._type)
            .position(Isometry {
                rotation: quat_to_rotation(transform.rotation),
                translation: vec3_to_translation(transform.position),
            })
            .build();
        let r_collider = ColliderBuilder::new(get_shared_shape(&pipeline, &transform.scale_matrix(), &collider))
            .friction(collider.friction)
            .restitution(collider.restitution)
            //.mass_properties(MassProperties::new(rapier3d::prelude::Point::new(0.0, 0.0, 0.0), 10.0, rapier3d::na::zero()))
            .build();

        // Add the collider and rigidbody
        let sim = &mut world.physics;
        let rigidbody_handle = sim.bodies.insert(r_rigibody);
        let collider_handle = sim.colliders.insert_with_parent(r_collider, rigidbody_handle, &mut sim.bodies);

        // Set the handles in their respective components
        let mut rigidbody = components.get_mut::<crate::components::RigidBody>().unwrap();
        rigidbody.handle = rigidbody_handle;
        let mut collider = components.get_mut::<crate::components::Collider>().unwrap();
        collider.handle = collider_handle;
    }
}

// Create the physics rigidbody & collider system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .link::<crate::components::RigidBody>()
        .link::<crate::components::Collider>()
        .link::<crate::components::Transform>()
        .with_added_entities_event(added_entities)
        .build();
}
