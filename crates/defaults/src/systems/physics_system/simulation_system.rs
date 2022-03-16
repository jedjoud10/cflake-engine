use rapier3d::na::Isometry;
use world::ecs::component::ComponentQueryParams;
use world::ecs::component::ComponentQuerySet;
use world::physics::PHYSICS_TIME_STEP;
use world::World;

use crate::components::{Collider, RigidBody, Transform};

use super::{quat_to_rotation, rotation_to_quat, vec3_to_translation, vec3_to_vector, vector_to_vec3};

// Run the physics simulation
fn run(world: &mut World, mut data: ComponentQuerySet) {
    // Execute only if we need to
    let physics = world.globals.get_mut::<crate::globals::Physics>().unwrap();
    let current_time = world.time.elapsed();
    if (current_time - physics.last_execution_time) > PHYSICS_TIME_STEP {
        physics.last_execution_time = current_time;
    } else {
        return;
    }
    physics.active_num = 0;

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
                r_rigidbody.set_position(isometry, true);
            }
        }
        // Check if we even need to update the attributes
        if components.was_mutated::<RigidBody>().unwrap_or_default() {
            let rigidbody = components.get::<RigidBody>().unwrap();
            // Update the Rapier3D rigibody
            if let Some(r_rigidbody) = world.physics.bodies.get_mut(rigidbody.handle) {
                r_rigidbody.set_linvel(vec3_to_vector(rigidbody.velocity), true);
                r_rigidbody.set_angvel(vec3_to_vector(rigidbody.angular_velocity), true);
                r_rigidbody.apply_force(vec3_to_vector(rigidbody.force), true);
            }
        }
        // Check if we even need to update the collider
        if components.was_mutated::<Collider>().unwrap_or_default() {
            let collider = components.get::<Collider>().unwrap();
            // Update the Rapier3D collider
            if let Some(r_collider) = world.physics.colliders.get_mut(collider.handle) {
                r_collider.set_friction(collider.material.friction);
                r_collider.set_restitution(collider.material.restitution);
                r_collider.set_friction_combine_rule(collider.material.friction_combine_rule);
                r_collider.set_restitution_combine_rule(collider.material.restitution_combine_rule);
            }
        }
    }

    // Step the simulation once
    world.physics.step();

    // After each step, we must update the components with their new values
    for (_, components) in query.iter_mut() {
        // Get the handle only
        let handle = components.get::<RigidBody>().unwrap().handle;
        if let Some(r_rigidbody) = world.physics.bodies.get(handle) {
            if !r_rigidbody.is_sleeping() {
                // Update the components
                let mut rigidbody = components.get_mut_silent::<RigidBody>().unwrap();
                rigidbody.velocity = vector_to_vec3(*r_rigidbody.linvel());
                rigidbody.angular_velocity = vector_to_vec3(*r_rigidbody.angvel());
                let mut transform = components.get_mut::<Transform>().unwrap();
                transform.position = vector_to_vec3(r_rigidbody.position().translation.vector);
                transform.rotation = rotation_to_quat(*r_rigidbody.rotation());
                physics.active_num += 1;
            }
        }
    }
}

// Create the physics simulation system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder(&mut world.events.ecs)
        .query(ComponentQueryParams::default().link::<RigidBody>().link::<Collider>().link::<Transform>())
        .event(run)
        .build();
}
