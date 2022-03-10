use world::physics::PHYSICS_TIME_STEP;
use world::World;
use world::{ecs::event::EventKey, physics::rapier3d::na::Isometry};

use super::{quat_to_rotation, rotation_to_quat, vec3_to_translation, vec3_to_vector, vector_to_vec3};

// Run the physics simulation
fn run(world: &mut World, mut data: EventKey) {
    // Execute only if we need to
    let physics = world.globals.get_mut::<crate::globals::Physics>().unwrap();
    let current_time = world.time.elapsed;
    if (current_time - physics.last_execution_time) > PHYSICS_TIME_STEP as f64 {
        physics.last_execution_time = current_time;
    } else {
        return;
    }

    // Update the position/rotation and attributes of each rigidbody since we might have externally updated them
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.iter() {
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
        // Check if we even need to update the attributes
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
    for (_, components) in query.iter_mut() {
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
        .systems
        .builder()
        .link::<crate::components::RigidBody>()
        .link::<crate::components::Collider>()
        .link::<crate::components::Transform>()
        .with_run_event(run)
        .build();
}
