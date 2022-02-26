use crate::systems::physics_system::{quat_to_rotation, vec3_to_translation};
use world::physics::rapier3d;
use world::physics::rapier3d::na::{Isometry, Point3};
use world::physics::rapier3d::prelude::{RigidBodyBuilder, SharedShape};
use world::rendering::basics::mesh::Mesh;
use world::World;
use world::{ecs::event::EventKey, physics::rapier3d::prelude::ColliderBuilder};

// Convert a rendering mesh to it's SharedShape counterpart
fn get_mesh(mesh: &Mesh) -> SharedShape {
    // Convert vertices and indices
    let vertices = mesh
        .vertices
        .positions
        .iter()
        .map(|vertex| Point3::new(vertex.x, vertex.y, vertex.z))
        .collect::<Vec<Point3<f32>>>();
    let indices = mesh
        .indices
        .chunks_exact(3)
        .map(|slice| slice.try_into().unwrap())
        .collect::<Vec<[u32; 3]>>();

    // Done
    SharedShape::trimesh(vertices, indices)
}

// Get the Rapier3D shared shape from a collider components
fn get_shared_shape(collider: &crate::components::Collider) -> SharedShape {
    match &collider._type {
        crate::components::ColliderType::Shape(shape) => todo!(),
        crate::components::ColliderType::Mesh(mesh) => get_mesh(mesh),
    }
}

// Whenever we add a rigidbody that has a collider attached to it, we must add them to the Rapier3D simulation
fn added_entities(world: &mut World, mut data: EventKey) {
    // Add each rigidbody and it's corresponding collider
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter_mut() {
        // Get the components
        let rigidbody = components
            .get_component::<crate::components::RigidBody>()
            .unwrap();
        let collider = components
            .get_component::<crate::components::Collider>()
            .unwrap();
        let transform = components
            .get_component::<crate::components::Transform>()
            .unwrap();

        // Transform to Rapier3D collider and rigibody
        let r_rigibody = RigidBodyBuilder::new(rigidbody._type)
            .position(Isometry {
                rotation: quat_to_rotation(transform.rotation),
                translation: vec3_to_translation(transform.position),
            })
            .build();
        //let r_collider = ColliderBuilder::new()
    }
}

// Create the physics rigidbody & collider system
pub fn system(world: &mut World) {
    world
        .ecs
        .build_system()
        .link::<crate::components::RigidBody>()
        .link::<crate::components::Collider>()
        .link::<crate::components::Transform>()
        .with_added_entities_event(added_entities)
        .build();
}
