use coords::{Position, Rotation};
use ecs::{Scene, added, modified};
use world::{System, World, post_user};
use crate::{RigidBody, Physics, SphereCollider};

// This will spawn in the required rapier counter-part of the components
fn pre_step_spawn_rapier_counterparts(physics: &mut Physics, scene: &mut Scene) {
    // Spawn in the RigidBody components
    let filter = added::<&RigidBody>();
    for rigid_body in scene.query_mut_with::<&mut RigidBody>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let _type = rigid_body._type;
        let rb = rapier3d::dynamics::RigidBodyBuilder::new(_type).build();
        let handle = physics.bodies.insert(rb);
        rigid_body.handle = Some(handle);
    }

    // Spawn in the Sphere collider components
    let filter = added::<&SphereCollider>();
    for (collider, rigid_body) in scene.query_mut_with::<(&mut SphereCollider, &RigidBody)>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let collider = rapier3d::geometry::ColliderBuilder::ball(0.5)
            .mass(10.0)
            .friction(0.2)
            .restitution(0.7)
            .density(collider.mass)
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }

    // Spawn in the Cuboid collider components
    

    // TODO: Spawn in the Mesh collider components

}

// This will de-spawn the required rapier counter-part of the components
fn pre_step_despawn_rapier_counterparts(physics: &mut Physics, scene: &mut Scene) {
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

    // Destroy removed RigidBody components
    for removed_rigid_body in scene.removed_mut::<RigidBody>() {
        if let Some(handle) = removed_rigid_body.handle {
            bodies.remove(
                handle,
                islands,
                colliders,
                impulse_joints,
                multibody_joints,
                false
            );
        }
    }

    // TODO: Destroy removed Sphere colliders
}

// This will synchronize the rapier counter-part to the data of the components
fn pre_step_sync_rapier_to_comps(physics: &mut Physics, scene: &mut Scene) {
    // Synchronize the RigidBody components 
    let filter = modified::<&RigidBody>();
    for (rigid_body, position, rotation) in scene.query_with::<(&RigidBody, &Position, &Rotation)>(filter) {
        if let Some(handle) = rigid_body.handle {
            let rb = physics.bodies.get_mut(handle).unwrap();
            rb.set_position(crate::trans_rot_to_isometry(**position, **rotation), false);
        }
    }

    // TODO: Synchronize the Sphere Collider components
}

// This will synchronize the component data to the newly computed rapier data
fn post_step_sync_comps_to_rapier(physics: &mut Physics, scene: &mut Scene) {
    for (rigid_body, position, rotation) in scene.query_mut::<(&mut RigidBody, &mut Position, &mut Rotation)>() {
        if let Some(handle) = rigid_body.handle {
            let rb = physics.bodies.get_mut(handle).unwrap();
            let (new_position, new_rotation) = crate::isometry_to_trans_rot(&rb.position());
            **position = new_position;
            **rotation = new_rotation;
        }
    }
}

// Synchronize the physics components before stepping
// This will spawn, despawn, and synchronize states of components
fn pre_step_tick_sync(world: &mut World) {
    let mut physics = world.get_mut::<Physics>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    pre_step_spawn_rapier_counterparts(&mut physics, &mut scene);
    pre_step_despawn_rapier_counterparts(&mut physics, &mut scene);
    pre_step_sync_rapier_to_comps(&mut physics, &mut scene);
}

// Synchronize the physics components after stepping
fn post_step_tick_sync(world: &mut World) {
    let mut physics = world.get_mut::<Physics>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    post_step_sync_comps_to_rapier(&mut physics, &mut scene);
}

// Will be responsible for syncing rapier <-> scene entities before stepping
pub fn pre_step_sync(system: &mut System) {
    system.insert_update(pre_step_tick_sync)
        .before(crate::systems::step::system)
        .after(post_user);
}

// Will be responsible for syncing rapier <-> scene entities after stepping
pub fn post_step_sync(system: &mut System) {
    system.insert_update(post_step_tick_sync).after(crate::systems::step::system)
        .after(post_user)
        .after(pre_step_sync);
}