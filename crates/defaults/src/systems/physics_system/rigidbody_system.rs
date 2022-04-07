use crate::components::{Collider, ColliderGeometry, RigidBody, Transform};
use crate::systems::physics_system::{quat_to_rotation, vec3_to_translation};
use rapier3d::na::{Isometry, Point3};
use rapier3d::prelude::{ColliderBuilder, MassProperties, RigidBodyBuilder, SharedShape};
use world::ecs::{Query, EntityState};
use world::math::shapes::ShapeType;
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Pipeline;
use world::World;
use super::vec3_to_point;

/*
// Convert a rendering mesh to it's SharedShape counterpart
fn get_mesh(scale_matrix: &vek::Mat4<f32>, mesh: &Mesh) -> SharedShape {
    // Convert vertices and indices
    let vertices = mesh
    .vertices()
    .positions
    .iter()
    // Scale the points by the scale matrix
    .map(|&vertex| scale_matrix.mul_point(vertex))
        .map(|vertex| Point3::new(vertex.x, vertex.y, vertex.z))
        .collect::<Vec<Point3<f32>>>();
        let indices = mesh.indices().chunks_exact(3).map(|slice| slice.try_into().unwrap()).collect::<Vec<[u32; 3]>>();
        // Done
        SharedShape::trimesh(vertices, indices)
    }
    */

// Get the Rapier3D shared shape from a collider components
fn get_shared_shape(pipeline: &Pipeline, scale_matrix: &vek::Mat4<f32>, collider: &Collider) -> SharedShape {
    match &collider.geometry {
        ColliderGeometry::Shape(shape) => match shape {
            ShapeType::Cuboid(cuboid) => SharedShape::cuboid(cuboid.size.x / 2.0, cuboid.size.y / 2.0, cuboid.size.z / 2.0),
            ShapeType::Sphere(sphere) => SharedShape::ball(sphere.radius),
            ShapeType::VerticalCapsule(capsule) => SharedShape::capsule(vec3_to_point(capsule.bottom()), vec3_to_point(capsule.top()), capsule.radius),
        },
        ColliderGeometry::Mesh { mesh, mass: _, com: _ } => todo!(),
    }
}

// Whenever we add a rigidbody that has a collider attached to it, we must add them to the Rapier3D simulation
fn run(world: &mut World) {
    // Add each rigidbody and it's corresponding collider
    let query = Query::new::<(&mut RigidBody, &mut Collider, &Transform)>(&mut world.ecs).unwrap();

    // Le physics simulation
    let sim = &mut world.physics;
    for (rigidbody, collider, transform) in query {
        // Transform to Rapier3D collider and rigibody
        let r_rigibody = RigidBodyBuilder::new(rigidbody._type)
            .position(Isometry {
                rotation: quat_to_rotation(transform.rotation),
                translation: vec3_to_translation(transform.position),
            })
            .build();
        let builder = ColliderBuilder::new(get_shared_shape(&world.pipeline, &transform.scale_matrix(), collider))
            .friction(collider.material.friction)
            .restitution(collider.material.restitution);

        // Set mass for mesh colliders manually
        let builder = if let Some((_, &mass, &com_offset)) = collider.geometry.as_mesh() {
            builder.mass_properties(MassProperties::new(vec3_to_point(com_offset), mass, rapier3d::na::zero()))
        } else {
            builder
        };

        // Build
        let r_collider = builder.build();

        // Add the collider and rigidbody
        let rigidbody_handle = sim.bodies.insert(r_rigibody);
        let collider_handle = sim.colliders.insert_with_parent(r_collider, rigidbody_handle, &mut sim.bodies);

        // Set the handles in their respective components
        rigidbody.handle = rigidbody_handle;
        collider.handle = collider_handle;
    }
    /*
    let removed = &mut data.get_mut(0).unwrap().delta.removed;
    // Also remove the rigidbodies that we don't need anymore
    for (_, components) in removed {
        let rb = components.get::<RigidBody>().unwrap();
        let collider = components.get::<Collider>().unwrap();
        sim.bodies.remove(rb.handle, &mut sim.islands, &mut sim.colliders, &mut sim.joints).unwrap();
        sim.colliders.remove(collider.handle, &mut sim.islands, &mut sim.bodies, false).unwrap();
    }
    */
}

// Create the physics rigidbody & collider system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
