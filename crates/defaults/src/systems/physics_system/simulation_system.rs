use world::ecs::component::ComponentQueryParameters;
use world::physics::PHYSICS_TIME_STEP;
use world::World;
use world::ecs::component::ComponentQuerySet;
use rapier3d::na::Isometry;

use crate::components::{Collider, RigidBody, Transform};

use super::{quat_to_rotation, rotation_to_quat, vec3_to_translation, vec3_to_vector, vector_to_vec3};

// Run the physics simulation
fn run(world: &mut World, mut data: ComponentQuerySet) {
    // Execute only if we need to
    let physics = world.globals.get_mut::<crate::globals::Physics>().unwrap();
    let current_time = world.time.elapsed;
    if (current_time - physics.last_execution_time) > PHYSICS_TIME_STEP as f64 {
        physics.last_execution_time = current_time;
    } else {
        return;
    }

    // Update the position/rotation and attributes of each rigidbody since we might have externally updated them
    let query = &mut data.get_mut(0).unwrap().all;
    for (_, components) in query.iter() {
        // Check if we even need to update the transform
        if components.was_mutated::<Transform>().unwrap_or_default() {
            let rigidbody = components.get::<RigidBody>().unwrap();
            let transform = components.get::<Transform>().unwrap();
            let isometry = Isometry {
                rotation: quat_to_rotation(transform.rotation),
                translation: vec3_to_translation(transform.position),
            };
            // Update the Rapier3D rigibody
            if let Some(r_rigidbody) = world.physics.bodies.get_mut(rigidbody.handle) {
                // TODO: Fix wake_up
                r_rigidbody.set_position(isometry, false);
                // TODO: Rigidbody forces
            }
        }
        // Check if we even need to update the attributes
        if components.was_mutated::<RigidBody>().unwrap_or_default() {
            let rigidbody = components.get::<RigidBody>().unwrap();
            // Update the Rapier3D rigibody
            if let Some(r_rigidbody) = world.physics.bodies.get_mut(rigidbody.handle) {
                r_rigidbody.set_linvel(vec3_to_vector(rigidbody.velocity), false);
                r_rigidbody.set_angvel(vec3_to_vector(rigidbody.angular_velocity), false);
            }
        }
        // Check if we even need to update the collider
        if components.was_mutated::<Collider>().unwrap_or_default() {
            
        }
    }

    // Step the simulation once
    world.physics.step();

    // After each step, we must update the components with their new values
    for (_, components) in query.iter_mut() {
        // Get the handle only
        let handle = components.get_mut::<RigidBody>().unwrap().handle;
        if let Some(r_rigidbody) = world.physics.bodies.get(handle) {
            if !r_rigidbody.is_sleeping() {
                // Update the components
                let mut rigidbody = components.get_mut::<RigidBody>().unwrap();
                rigidbody.velocity = vector_to_vec3(*r_rigidbody.linvel());
                rigidbody.angular_velocity = vector_to_vec3(*r_rigidbody.angvel());
                let mut transform = components.get_mut::<Transform>().unwrap();
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
        .query(ComponentQueryParameters::default().link::<RigidBody>().link::<Collider>().link::<Transform>())
        .event(run)
        .build();
}
