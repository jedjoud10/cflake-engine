use rapier3d::prelude::*;
use world::{post_user, System, World};

use crate::Physics;


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
    /*
    system.insert_update(update)
        .after(post_user)
        .before(graphics::present)
        .before(rendering::systems::rendering::system)
        .before(rendering::systems::matrix::system);
    */
}