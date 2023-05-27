use coords::{Position, Rotation};
use ecs::{Scene, added, modified};
use world::{System, World, post_user};
use crate::{RigidBody, Physics, SphereCollider, CuboidCollider, AngularVelocity, Velocity};

// This will spawn in the required rapier counter-part of the components
fn pre_step_spawn_rapier_counterparts(physics: &mut Physics, scene: &mut Scene) {
    // Spawn in the RigidBody components
    let filter = added::<&RigidBody>();
    for rigid_body in scene.query_mut_with::<&mut RigidBody>(filter) {
        if rigid_body.handle.is_some() {
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

        let collider = rapier3d::geometry::ColliderBuilder::ball(collider.radius)
            .mass(collider.mass)
            .friction(collider.friction)
            .restitution(collider.restitution)
            .density(collider.mass)
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }

    // Spawn in the Cuboid collider components
    let filter = added::<&CuboidCollider>();
    for (collider, rigid_body) in scene.query_mut_with::<(&mut CuboidCollider, &RigidBody)>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let collider = rapier3d::geometry::ColliderBuilder::cuboid(collider.half_extent.w, collider.half_extent.h, collider.half_extent.d)
            .mass(collider.mass)
            .friction(collider.friction)
            .restitution(collider.restitution)
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }
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

    // Destroy removed Sphere Collider components
    for removed_sphere_colliders in scene.removed_mut::<SphereCollider>() {
        if let Some(handle) = removed_sphere_colliders.handle {
            colliders.remove(
                handle,
                islands,
                bodies,
                true,
            );
        }
    }

    // Destroy removed Cuboid Collider components
    for removed_cuboid_colliders in scene.removed_mut::<CuboidCollider>() {
        if let Some(handle) = removed_cuboid_colliders.handle {
            colliders.remove(
                handle,
                islands,
                bodies,
                true,
            );
        }
    }
}

// This will synchronize the rapier counter-part to the data of the components
fn pre_step_sync_rapier_to_comps(physics: &mut Physics, scene: &mut Scene) {
    // Filter for the rigidbody query
    let filter = modified::<&RigidBody>() |
        modified::<&Position>() |
        modified::<&Rotation>() |
        modified::<&AngularVelocity>() |
        modified::<&Velocity>();

    // Synchronize the RigidBody components 
    for (rigid_body, position, rotation, velocity, angular_velocity) in scene.query_with::<(&RigidBody, &Position, &Rotation, &Velocity, &AngularVelocity)>(filter) {
        if let Some(handle) = rigid_body.handle {
            let rb = physics.bodies.get_mut(handle).unwrap();
            rb.set_position(crate::trans_rot_to_isometry(**position, **rotation), false);
            rb.set_linvel(crate::vek_vec_to_na_vec(**velocity), false);
            rb.set_angvel(crate::vek_vec_to_na_vec(**angular_velocity), false);
        }
    }

    // Synchronize the Sphere Collider components
    let filter = modified::<&SphereCollider>();
    for sphere_collider in scene.query_with::<&SphereCollider>(filter) {
        if let Some(handle) = sphere_collider.handle {
            let collider = physics.colliders.get_mut(handle).unwrap();
            let ball = collider.shape_mut().as_ball_mut().unwrap();
            ball.radius = sphere_collider.radius;
            collider.set_friction(sphere_collider.friction);
            collider.set_restitution(sphere_collider.restitution);
        }
    }

    // Synchronize the Cuboid Collider components
    let filter = modified::<&CuboidCollider>();
    for cuboid_collider in scene.query_with::<&CuboidCollider>(filter) {
        if let Some(handle) = cuboid_collider.handle {
            let collider = physics.colliders.get_mut(handle).unwrap();
            let ball = collider.shape_mut().as_cuboid_mut().unwrap();
            ball.half_extents.x = cuboid_collider.half_extent.w;
            ball.half_extents.y = cuboid_collider.half_extent.h;
            ball.half_extents.z = cuboid_collider.half_extent.d;
            collider.set_friction(cuboid_collider.friction);
            collider.set_restitution(cuboid_collider.restitution);
        }
    }
}

// This will synchronize the component data to the newly computed rapier data
fn post_step_sync_comps_to_rapier(physics: &mut Physics, scene: &mut Scene) {
    for (rigid_body, position, rotation, velocity, angular_velocity) in scene.query_mut::<(&mut RigidBody, &mut Position, &mut Rotation, &mut Velocity, &mut AngularVelocity)>() {
        if let Some(handle) = rigid_body.handle {
            if rigid_body._type.is_dynamic() {
                let rb = physics.bodies.get_mut(handle).unwrap();
                let (new_position, new_rotation) = crate::isometry_to_trans_rot(&rb.position());
                let new_velocity = crate::na_vec_to_vek_vec(*rb.linvel());
                let new_angular_velocity = crate::na_vec_to_vek_vec(*rb.angvel());
                **position = new_position;
                **rotation = new_rotation;
                **velocity = new_velocity;
                **angular_velocity = new_angular_velocity;
            }
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
    system.insert_tick(pre_step_tick_sync)
        .before(crate::systems::step::system)
        .after(post_user);
}

// Will be responsible for syncing rapier <-> scene entities after stepping
pub fn post_step_sync(system: &mut System) {
    system.insert_tick(post_step_tick_sync)
        .after(crate::systems::step::system)
        .after(post_user)
        .after(pre_step_sync);
}