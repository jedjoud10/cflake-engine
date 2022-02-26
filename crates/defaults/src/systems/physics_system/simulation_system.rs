use world::World;
use world::{ecs::event::EventKey, physics::rapier3d::na::Isometry};

use super::{quat_to_rotation, vec3_to_translation, vec3_to_vector, vector_to_vec3, rotation_to_quat};

// Run the physics simulation
fn run(world: &mut World, mut data: EventKey) {
    // Update the position/rotation and velocity of each rigidbody since we might have externally updated them
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter() {
        // Check if we even need to update the position/rotation
        if components.was_mutated::<crate::components::Transform>().unwrap_or_default() {
            let rigidbody = components.get::<crate::components::RigidBody>().unwrap();
            let transform = components.get::<crate::components::Transform>().unwrap();
            let isometry = Isometry {
                rotation: quat_to_rotation(transform.rotation),
                translation: vec3_to_translation(transform.position),
            };
            // Update the Rapier3D rigibody
            if let Some(r_rigidbody) = world.physics.bodies.get_mut(rigidbody.handle) {
                r_rigidbody.set_position(isometry, true);
            }
        }
        // Check if we even need to update the velocity
        if components.was_mutated::<crate::components::RigidBody>().unwrap_or_default() {
            let rigidbody = components.get::<crate::components::RigidBody>().unwrap();
            // Update the Rapier3D rigibody
            if let Some(r_rigidbody) = world.physics.bodies.get_mut(rigidbody.handle) {
                r_rigidbody.set_linvel(vec3_to_vector(rigidbody.velocity), true);
            }
        }
    }

    // Step the simulation once
    world.physics.step();

    // After each step, we must update the components with their new values
    for (_, components) in query.write().iter_mut() {
        // Get the handle only
        let handle = components.get_mut::<crate::components::RigidBody>().unwrap().handle;
        if let Some(r_rigidbody) = world.physics.bodies.get(handle) {
            if !r_rigidbody.is_sleeping() {
                // Update the components
                let mut rigidbody = components.get_mut::<crate::components::RigidBody>().unwrap();
                rigidbody.velocity = vector_to_vec3(*r_rigidbody.linvel());
                let mut transform = components.get_mut::<crate::components::Transform>().unwrap();
                transform.position = vector_to_vec3(r_rigidbody.position().translation.vector);
                transform.rotation = rotation_to_quat(*r_rigidbody.rotation());
            }
        }
    }
}

// Create the physics simulation system
pub fn system(world: &mut World) {
    world
        .ecs
        .build_system()
        .link::<crate::components::RigidBody>()
        .link::<crate::components::Collider>()
        .link::<crate::components::Transform>()
        .with_run_fixed_event(run)
        .build();
}
