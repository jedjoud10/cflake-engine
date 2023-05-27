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
fn tick(world: &mut World) {
    let mut physics = world.get_mut::<Physics>().unwrap();

    let Physics {
        bodies,
        colliders,
        integration_parameters,
        physics_pipeline,
        islands,
        broad_phase,
        narrow_phase,
        impulse_joints,
        multibody_joints,
        ccd_solver,
    } = &mut *physics;

    let gravity = vector![0.0, -9.81, 0.0];

    for x in 0..2 {
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            islands,
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            None,
            &(),
            &(),
        );
    }
}


// Create the main physics system that will be responsible for stepping through the Rapier simulation
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_tick(tick)
        .after(post_user)
        .before(ecs::post_frame_or_tick);
}