use coords::Position;
use ecs::Scene;
use rapier3d::prelude::*;
use utils::Time;
use world::{post_user, System, World, user};
use crate::{Physics, Velocity};

// Creates the physics resource and add it into the world
fn init(world: &mut World) {
    let physics = Physics::new();
    world.insert(physics);
}

// Step through the physics simulation
fn update(world: &mut World) {
    let mut physics = world.get_mut::<Physics>().unwrap();

    let Physics {
        rigid_body_set,
        collider_set,
        integration_parameters,
        physics_pipeline,
        island_manager,
        broad_phase,
        narrow_phase,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
    } = &mut *physics;

    let gravity = vector![0.0, -9.81, 0.0];

    physics_pipeline.step(
        &gravity,
        &integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        None,
        &(),
        &(),
      );
}


// Create the main physics system that will be responsible for simulating physics using rapier 
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update)
        .after(post_user);
}