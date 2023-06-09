use rapier3d::prelude::*;
use utils::{Time, Storage};
use coords::{Position, Rotation};
use ecs::{Scene, added, modified, Entity};
use world::{System, World, post_user, user};
use crate::{RigidBody, Physics, SphereCollider, CuboidCollider, AngularVelocity, Velocity, MeshCollider, PhysicsSurface, CapsuleCollider};
use crate::{LastTickedVelocity, LastTickedAngularVelocity, CurrentTickedVelocity, CurrentTickedAngularVelocity};
use coords::{LastTickedPosition, LastTickedRotation, CurrentTickedPosition, CurrentTickedRotation};

// This will spawn in the required rapier counter-part of the components
fn pre_step_spawn_rapier_counterparts(physics: &mut Physics, scene: &mut Scene) {
    // Spawn in the RigidBody components (and keep track of the new entities)
    let filter = added::<&RigidBody>();
    let mut interpolated_entities = Vec::<Entity>::new();
    for (entity, rigid_body) in scene.query_mut_with::<(&Entity, &mut RigidBody)>(filter) {
        if rigid_body.handle.is_some() {
            continue;
        };

        let _type = rigid_body._type;
        let rb = rapier3d::dynamics::RigidBodyBuilder::new(_type).build();
        let handle = physics.bodies.insert(rb);
        rigid_body.handle = Some(handle);

        if _type == RigidBodyType::Dynamic && rigid_body.interpolated {
            interpolated_entities.push(*entity);
        }
    }

    // Spawn in the Sphere collider components
    let filter = added::<&SphereCollider>();
    for (collider, rigid_body) in scene.query_mut_with::<(&mut SphereCollider, &RigidBody)>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let collider = rapier3d::geometry::ColliderBuilder::ball(collider.radius)
            .mass(collider.mass)
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
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }

    // Spawn in the Capsule collider components
    let filter = added::<&CapsuleCollider>();
    for (collider, rigid_body) in scene.query_mut_with::<(&mut CapsuleCollider, &RigidBody)>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let collider = rapier3d::geometry::ColliderBuilder::capsule_y(collider.height / 2.0, collider.radius)
            .mass(collider.mass)
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }

    // Spawn in the Mesh collider components
    let filter = added::<&MeshCollider>();
    for (collider, rigid_body) in scene.query_mut_with::<(&mut MeshCollider, &RigidBody)>(filter) {
        let Some(handle) = rigid_body.handle else {
            continue;
        };

        let (vertices, triangles) = match collider.inner.take().unwrap() {
            crate::InnerMeshCollider::ExplicitOwned { vertices, triangles } => {
                let vertices: Vec<rapier3d::na::Point3<f32>> = vertices.into_iter().map(|x| crate::vek_vec_to_na_point(x)).collect::<_>();
                (vertices, triangles)
            },
        };

        let collider = rapier3d::geometry::ColliderBuilder::trimesh(vertices, triangles)
            .mass(collider.mass)
            .build();
        physics.colliders.insert_with_parent(collider, handle, &mut physics.bodies);
    }

    // Automatically add the ticked coord components
    for entity in interpolated_entities {
        let mut entry = scene.entry_mut(entity).unwrap();
        
        let query = entry.as_query::<(&Position, &Rotation, &Velocity, &AngularVelocity)>();

        if let Some((
            position,
            rotation,
            velocity,
            angular_velocity
        )) = query {
            entry.insert((
                LastTickedPosition::from(**position),
                LastTickedRotation::from(**rotation),
                LastTickedVelocity::from(**velocity),
                LastTickedAngularVelocity::from(**angular_velocity),
                CurrentTickedPosition::from(**position),
                CurrentTickedRotation::from(**rotation),
                CurrentTickedVelocity::from(**velocity),
                CurrentTickedAngularVelocity::from(**angular_velocity),
            )).unwrap();
        }
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
fn pre_step_sync_rapier_to_comps(physics: &mut Physics, scene: &mut Scene, surfaces: &Storage<PhysicsSurface>) {
    let query = scene.query_mut::<(&mut RigidBody, &Position, &Rotation, &Velocity, &AngularVelocity)>();
    for (rigid_body, position, rotation, velocity, angular_velocity) in query {
        if let Some(handle) = rigid_body.handle {
            let rb = physics.bodies.get_mut(handle).unwrap();
            rb.set_position(crate::trans_rot_to_isometry(**position, **rotation), false);
            rb.set_linvel(crate::vek_vec_to_na_vec(**velocity), false);
            rb.set_angvel(crate::vek_vec_to_na_vec(**angular_velocity), false);

            for force in rigid_body.forces.drain(..) {
                rb.add_force(crate::vek_vec_to_na_vec(force), true);
            }

            for torque in rigid_body.torques.drain(..) {
                rb.add_torque(crate::vek_vec_to_na_vec(torque), true);
            }

            for (force, point) in rigid_body.forces_at_points.drain(..) {
                let force = crate::vek_vec_to_na_vec(force);
                let point = crate::vek_vec_to_na_point(point);
                rb.add_force_at_point(force, point, true);
            }

            for impulse in rigid_body.impulses.drain(..) {
                rb.apply_impulse(crate::vek_vec_to_na_vec(impulse), true);
            }

            for torque_impulse in rigid_body.torque_impulses.drain(..) {
                rb.apply_torque_impulse(crate::vek_vec_to_na_vec(torque_impulse), true);
            }

            for (impulse, point) in rigid_body.impulses_at_points.drain(..) {
                let impulse = crate::vek_vec_to_na_vec(impulse);
                let point = crate::vek_vec_to_na_point(point);
                rb.apply_impulse_at_point(impulse, point, true);
            }
        }
    }

    // Synchronize the Sphere Collider components
    let filter = modified::<&SphereCollider>();
    for sphere_collider in scene.query_with::<&SphereCollider>(filter) {
        if let Some(handle) = sphere_collider.handle {
            let collider = physics.colliders.get_mut(handle).unwrap();
            let ball = collider.shape_mut().as_ball_mut().unwrap();
            ball.radius = sphere_collider.radius;

            let physics_surface = sphere_collider.material.as_ref().map(|x| surfaces[x]).unwrap_or_default();
            collider.set_friction(physics_surface.friction);
            collider.set_restitution(physics_surface.restitution);
        }
    }

    // Synchronize the Cuboid Collider components
    let filter = modified::<&CuboidCollider>();
    for cuboid_collider in scene.query_with::<&CuboidCollider>(filter) {
        if let Some(handle) = cuboid_collider.handle {
            let collider = physics.colliders.get_mut(handle).unwrap();
            let cuboid = collider.shape_mut().as_cuboid_mut().unwrap();
            cuboid.half_extents.x = cuboid_collider.half_extent.w;
            cuboid.half_extents.y = cuboid_collider.half_extent.h;
            cuboid.half_extents.z = cuboid_collider.half_extent.d;

            let physics_surface = cuboid_collider.material.as_ref().map(|x| surfaces[x]).unwrap_or_default();
            collider.set_friction(physics_surface.friction);
            collider.set_restitution(physics_surface.restitution);
        }
    }

    // Synchronize the Capsule Collider components
    let filter: ecs::Wrap<ecs::Modified<&CapsuleCollider>> = modified::<&CapsuleCollider>();
    for capsule_collider in scene.query_with::<&CapsuleCollider>(filter) {
        if let Some(handle) = capsule_collider.handle {
            let collider = physics.colliders.get_mut(handle).unwrap();
            let capsule = collider.shape_mut().as_capsule_mut().unwrap();
            capsule.segment.a = crate::util::vek_vec_to_na_point(vek::Vec3::unit_y() * (-capsule_collider.height / 2.0));
            capsule.segment.b = crate::util::vek_vec_to_na_point(vek::Vec3::unit_y() * (capsule_collider.height / 2.0));
            capsule.radius = capsule_collider.radius;

            let physics_surface = capsule_collider.material.as_ref().map(|x| surfaces[x]).unwrap_or_default();
            collider.set_friction(physics_surface.friction);
            collider.set_restitution(physics_surface.restitution);
        }
    }
}

// Checks all the character controllers in the world and updates them
fn post_step_update_character_controllers(physics: &mut Physics, scene: &mut Scene) {
}

// Creates the physics resource and add it into the world
fn init(world: &mut World) {
    let physics = Physics::new();
    world.insert(physics);
    world.insert(Storage::<PhysicsSurface>::default());
}

// Step through the physics simulation
fn tick(world: &mut World) {
    let mut _physics = world.get_mut::<Physics>().unwrap();
    let mut _scene = world.get_mut::<Scene>().unwrap();
    let surfaces = world.get::<Storage<PhysicsSurface>>().unwrap();
    let time = world.get::<Time>().unwrap();
    let physics = &mut *_physics;
    let scene = &mut * _scene;

    // Executed before the physics step
    pre_step_spawn_rapier_counterparts(physics, scene);
    pre_step_despawn_rapier_counterparts(physics, scene);
    pre_step_sync_rapier_to_comps(physics, scene, &surfaces);

    // Swap next tick with current tick
    if time.tick_count() > 0 {
        let query = scene.query_mut::<(
            &mut RigidBody,
            &mut LastTickedPosition,
            &mut LastTickedRotation,
            &mut LastTickedVelocity,
            &mut LastTickedAngularVelocity,
            &CurrentTickedPosition,
            &CurrentTickedRotation,
            &CurrentTickedVelocity,
            &CurrentTickedAngularVelocity,
        )>();
    
        for (
            rigid_body,
            last_position,
            last_rotation,
            last_velocity,
            last_angular_velocity,
            next_position,
            next_rotation,
            next_velocity,
            next_angular_velocity,
        ) in query {
            if rigid_body._type.is_dynamic() {
                **last_position = **next_position;
                **last_rotation = **next_rotation;
                **last_velocity = **next_velocity;
                **last_angular_velocity = **next_angular_velocity;
            }
        }
    }

    // Step through the physics simulation each tick
    physics.step();

    // Update character controller rigid-bodies
    post_step_update_character_controllers(physics, scene);


    fn set_sub_tick_coords_type<TimeFrame: 'static>(scene: &mut Scene, bodies: &mut RigidBodySet, interpolated: bool) {
        let query = scene.query_mut::<(
            &mut RigidBody,
            &mut coords::UnmarkedPosition<coords::Global<TimeFrame>>,
            &mut coords::UnmarkedRotation<coords::Global<TimeFrame>>,
            &mut crate::UnmarkedVelocity<coords::Global<TimeFrame>>,
            &mut crate::UnmarkedAngularVelocity<coords::Global<TimeFrame>>
        )>();
        for (
            rigid_body,
            position,
            rotation,
            velocity,
            angular_velocity
        ) in query {
            if let Some(handle) = rigid_body.handle {
                if rigid_body._type.is_dynamic() && (rigid_body.interpolated == interpolated) {
                    let rb = bodies.get_mut(handle).unwrap();
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

    // Update sleeping state
    for rigid_body in scene.query_mut::<&mut RigidBody>() {
        if let Some(handle) = rigid_body.handle {
            if rigid_body._type.is_dynamic() {
                let rb = physics.bodies.get_mut(handle).unwrap();
                rigid_body.sleeping = rb.is_sleeping();
            }
        }
    }

    // Update current tick coordinates and frame to frame coords (if interpolation is disabled)
    set_sub_tick_coords_type::<coords::CurrentTick>(scene, &mut physics.bodies, true);
    set_sub_tick_coords_type::<coords::FrameToFrame>(scene, &mut physics.bodies, false);
}

// Sub tick interpolation for rigidbodies
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let time = world.get::<Time>().unwrap();
    let t = time.tick_interpolation();

    let query = scene.query_mut::<(
        &mut RigidBody,
        &LastTickedPosition,
        &LastTickedRotation,
        &LastTickedVelocity,
        &LastTickedAngularVelocity,
        &CurrentTickedPosition,
        &CurrentTickedRotation,
        &CurrentTickedVelocity,
        &CurrentTickedAngularVelocity,
        &mut Position,
        &mut Rotation,
        &mut Velocity,
        &mut AngularVelocity,
    )>();

    for (
        rigid_body,
        last_position,
        last_rotation,
        last_velocity,
        last_angular_velocity,
        next_position,
        next_rotation,
        next_velocity,
        next_angular_velocity,
        current_position,
        current_rotation,
        current_velocity,
        current_angular_velocity
    ) in query {
        if rigid_body._type.is_dynamic() && rigid_body.interpolated {
            **current_position = vek::Lerp::lerp(**last_position, **next_position, t);
            **current_rotation = vek::Slerp::slerp(**last_rotation, **next_rotation, t);
            **current_velocity = vek::Lerp::lerp(**last_velocity, **next_velocity, t);
            **current_angular_velocity = vek::Lerp::lerp(**last_angular_velocity, **next_angular_velocity, t);
        }
    }
}


// Create the main physics system that will be responsible for stepping through the Rapier simulation
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_tick(tick)
        .after(post_user)
        .before(ecs::post_frame_or_tick);
    system.insert_update(update)
        .after(post_user)
        .before(ecs::post_frame_or_tick)
        .before(rendering::systems::rendering::system);
}