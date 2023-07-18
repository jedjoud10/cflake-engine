use crate::{
    AngularVelocity, CapsuleCollider, CharacterController, CuboidCollider, GenericCollider,
    MeshCollider, Physics, PhysicsSurface, RigidBody, SphereCollider, Velocity,
};
use crate::{
    CurrentTickedAngularVelocity, CurrentTickedVelocity, LastTickedAngularVelocity,
    LastTickedVelocity,
};
use coords::{
    CurrentTickedPosition, CurrentTickedRotation, LastTickedPosition, LastTickedRotation,
};
use coords::{Position, Rotation};
use ecs::{added, Component, Entity, Scene};
use rapier3d::prelude::*;
use utils::{Storage, Time};
use world::{post_user, user, System, World};

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
        let rb = rapier3d::dynamics::RigidBodyBuilder::new(_type)
            .user_data(entity.to_raw() as u128)
            .locked_axes(rigid_body.locked)
            .build();
        log::trace!("create rapier rigidbody type");
        let handle = physics.bodies.insert(rb);
        rigid_body.handle = Some(handle);

        if !_type.is_fixed() && rigid_body.interpolated {
            interpolated_entities.push(*entity);
        }
    }

    fn insert_rapier_collider<C: GenericCollider + Component>(
        scene: &mut Scene,
        colliders: &mut ColliderSet,
        bodies: &mut RigidBodySet,
    ) {
        let filter = added::<&C>();
        for (entity, component, rigid_body) in
            scene.query_mut_with::<(&Entity, &mut C, &RigidBody)>(filter)
        {
            let Some(handle) = rigid_body.handle else {
                continue;
            };

            if let Some(collider) = C::build_collider(component, entity) {
                log::trace!("create rapier generic collider type");
                let handle = colliders.insert_with_parent(collider, handle, bodies);
                C::set_handle(component, handle);
            }
        }
    }

    let Physics {
        bodies, colliders, ..
    } = &mut *physics;

    // Insert the rapier colliders
    insert_rapier_collider::<SphereCollider>(scene, colliders, bodies);
    insert_rapier_collider::<CuboidCollider>(scene, colliders, bodies);
    insert_rapier_collider::<CapsuleCollider>(scene, colliders, bodies);
    insert_rapier_collider::<MeshCollider>(scene, colliders, bodies);

    // Automatically add the ticked coord components
    for entity in interpolated_entities {
        let mut entry = scene.entry_mut(entity).unwrap();

        let query = entry.as_query::<(&Position, &Rotation, &Velocity, &AngularVelocity)>();

        if let Some((position, rotation, velocity, angular_velocity)) = query {
            log::trace!("insert ticked components for interpolated physics entity");
            entry
                .insert((
                    LastTickedPosition::from(**position),
                    LastTickedRotation::from(**rotation),
                    LastTickedVelocity::from(**velocity),
                    LastTickedAngularVelocity::from(**angular_velocity),
                    CurrentTickedPosition::from(**position),
                    CurrentTickedRotation::from(**rotation),
                    CurrentTickedVelocity::from(**velocity),
                    CurrentTickedAngularVelocity::from(**angular_velocity),
                ))
                .unwrap();
        }
    }
}

// This will de-spawn the required rapier counter-part of the components
fn pre_step_despawn_rapier_counterparts(physics: &mut Physics, scene: &mut Scene) {
    let Physics {
        bodies,
        colliders,
        islands,
        impulse_joints,
        multibody_joints,
        ..
    } = &mut *physics;

    fn destroy_removed_collider<C: GenericCollider + Component>(
        scene: &mut Scene,
        colliders: &mut ColliderSet,
        islands: &mut IslandManager,
        bodies: &mut RigidBodySet,
    ) {
        for removed in scene.removed_mut::<C>() {
            if let Some(handle) = removed.handle() {
                colliders.remove(handle, islands, bodies, true);
            }
        }
    }

    // Destroy removed RigidBody components
    for removed_rigid_body in scene.removed_mut::<RigidBody>() {
        if let Some(handle) = removed_rigid_body.handle {
            bodies.remove(
                handle,
                islands,
                colliders,
                impulse_joints,
                multibody_joints,
                false,
            );
        }
    }

    // Destroy the rapier colliders
    destroy_removed_collider::<SphereCollider>(scene, colliders, islands, bodies);
    destroy_removed_collider::<CuboidCollider>(scene, colliders, islands, bodies);
    destroy_removed_collider::<CapsuleCollider>(scene, colliders, islands, bodies);
    destroy_removed_collider::<MeshCollider>(scene, colliders, islands, bodies);
}

// This will synchronize the rapier counter-part to the data of the components
fn pre_step_sync_rapier_to_comps(
    physics: &mut Physics,
    scene: &mut Scene,
    surfaces: &Storage<PhysicsSurface>,
) {
    let query = scene.query_mut::<(
        &mut RigidBody,
        Option<&Position>,
        Option<&Rotation>,
        Option<&Velocity>,
        Option<&AngularVelocity>,
    )>();
    for (rigid_body, position, rotation, velocity, angular_velocity) in query {
        if let Some(handle) = rigid_body.handle {
            let rb = physics.bodies.get_mut(handle).unwrap();

            let position = position.cloned().unwrap_or_default();
            let rotation = rotation.cloned().unwrap_or_default();
            rb.set_position(crate::trans_rot_to_isometry(*position, *rotation), false);

            if rigid_body._type.is_fixed() {
                continue;
            }

            rb.reset_forces(false);
            rb.reset_torques(false);

            let velocity = velocity.cloned().unwrap_or_default();
            let angular_velocity = angular_velocity.cloned().unwrap_or_default();
            rb.set_linvel(crate::vek_vec_to_na_vec(*velocity), false);
            rb.set_angvel(crate::vek_vec_to_na_vec(*angular_velocity), false);

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

    // Common function that will update the inner rapier data of colliders
    fn update_rapier_colliders<C: Component + GenericCollider>(
        scene: &mut Scene,
        surfaces: &Storage<PhysicsSurface>,
        physics: &mut Physics,
    ) {
        for (entity, component, rb) in scene.query_mut::<(&Entity, &mut C, &RigidBody)>() {
            if C::modified(component).take() {
                if let Some(handle) = C::handle(component) {
                    let physics_surface = C::material(component)
                        .as_ref()
                        .map(|x| surfaces[x])
                        .unwrap_or_default();
                    let collider = physics.colliders.get_mut(handle).unwrap();
                    collider.set_friction(physics_surface.friction);
                    collider.set_restitution(physics_surface.restitution);
                    collider.set_mass(C::mass(component));
                    let custom = C::cast_rapier_collider(collider);

                    if C::regenerate_when_updating() {
                        C::set_custom_rapier_collider_settings(component, custom);
                    } else if let Some(new_collider) = C::build_collider(component, entity) {
                        *collider = new_collider;
                    }
                } else if C::regenerate_when_updating() {
                    log::trace!("rebuilding collider...");
                    if let Some(collider) = C::build_collider(component, entity) {
                        log::trace!("rebuilt collider successfully");

                        let handle = physics.colliders.insert_with_parent(
                            collider,
                            rb.handle.unwrap(),
                            &mut physics.bodies,
                        );
                        C::set_handle(component, handle)
                    }
                }
            }
        }
    }

    // Update the rapier colliders
    update_rapier_colliders::<SphereCollider>(scene, surfaces, physics);
    update_rapier_colliders::<CuboidCollider>(scene, surfaces, physics);
    update_rapier_colliders::<CapsuleCollider>(scene, surfaces, physics);
    update_rapier_colliders::<MeshCollider>(scene, surfaces, physics);
}

// Checks all the character controllers in the world and updates them
fn post_step_update_character_controllers(_physics: &mut Physics, scene: &mut Scene) {
    for (cc, _position, _rotation, rb, velocity) in scene.query_mut::<(
        &mut CharacterController,
        &Position,
        &Rotation,
        &mut RigidBody,
        &mut Velocity,
    )>() {
        let direction = cc.direction.try_normalized().unwrap_or_default();

        // Current velocity and wished velocity
        let current = vek::Vec2::new(velocity.x, velocity.z);
        let wished = vek::Vec2::new(direction.x, direction.z) * cc.max_speed;
        let force = (wished - current) * cc.acceleration;
        rb.apply_force(vek::Vec3::new(force.x, 0.0, force.y));

        // Make the character controller jump if needed
        if std::mem::take(&mut cc.jumping) {
            rb.apply_impulse(vek::Vec3::unit_y() * cc.jump_force);
        }
    }
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
    let scene = &mut *_scene;

    // Executed before the physics step
    pre_step_spawn_rapier_counterparts(physics, scene);
    pre_step_despawn_rapier_counterparts(physics, scene);

    // Update character controller rigid-bodies
    post_step_update_character_controllers(physics, scene);

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
        ) in query
        {
            if !rigid_body._type.is_fixed() {
                **last_position = **next_position;
                **last_rotation = **next_rotation;
                **last_velocity = **next_velocity;
                **last_angular_velocity = **next_angular_velocity;
            }
        }
    }

    // Step through the physics simulation each tick
    physics.step();

    fn set_sub_tick_coords_type<TimeFrame: 'static>(
        scene: &mut Scene,
        bodies: &mut RigidBodySet,
        interpolated: bool,
    ) {
        let query = scene.query_mut::<(
            &mut RigidBody,
            &mut coords::UnmarkedPosition<coords::Global<TimeFrame>>,
            &mut coords::UnmarkedRotation<coords::Global<TimeFrame>>,
            &mut crate::UnmarkedVelocity<coords::Global<TimeFrame>>,
            &mut crate::UnmarkedAngularVelocity<coords::Global<TimeFrame>>,
        )>();
        for (rigid_body, position, rotation, velocity, angular_velocity) in query {
            if let Some(handle) = rigid_body.handle {
                if !rigid_body._type.is_fixed() && (rigid_body.interpolated == interpolated) {
                    let rb = bodies.get_mut(handle).unwrap();
                    let (new_position, new_rotation) = crate::isometry_to_trans_rot(rb.position());
                    let new_velocity = crate::na_vec_to_vek_vec(*rb.linvel());
                    let new_angular_velocity = crate::na_vec_to_vek_vec(*rb.angvel());
                    **position = new_position;
                    **rotation = new_rotation;
                    **velocity = new_velocity;
                    **angular_velocity = new_angular_velocity;
                    rigid_body.sleeping = rb.is_sleeping();
                }
            }
        }
    }

    // Update sleeping state
    for rigid_body in scene.query_mut::<&mut RigidBody>() {
        if let Some(handle) = rigid_body.handle {
            if !rigid_body._type.is_fixed() {
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
        current_angular_velocity,
    ) in query
    {
        if !rigid_body._type.is_fixed() && rigid_body.interpolated {
            **current_position = vek::Lerp::lerp(**last_position, **next_position, t);
            **current_rotation = vek::Slerp::slerp(**last_rotation, **next_rotation, t);
            **current_velocity = vek::Lerp::lerp(**last_velocity, **next_velocity, t);
            **current_angular_velocity =
                vek::Lerp::lerp(**last_angular_velocity, **next_angular_velocity, t);
        }
    }
}

// Create the main physics system that will be responsible for stepping through the Rapier simulation
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system
        .insert_tick(tick)
        .after(post_user)
        .after(ecs::pre_frame_or_tick)
        .before(ecs::post_frame_or_tick);
    system
        .insert_update(update)
        .after(post_user)
        .after(ecs::pre_frame_or_tick)
        .before(ecs::post_frame_or_tick)
        .before(rendering::systems::rendering::system);
}
