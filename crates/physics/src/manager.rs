use std::cell::RefCell;
use std::rc::Rc;

use rapier3d::data::Arena;
use rapier3d::math::{Rotation, Vector};
use rapier3d::na::{Quaternion, Unit, UnitQuaternion};
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyType, SharedShape};
use crate::collider::{Collider, ColliderType};
use crate::identifier::{PhysicsID};
use crate::rigidbody::*;
use crate::simulation::PhysicsSimulation;
use crate::surface::Surface;

// Physics manager that contains everything related to the Rapier3D physics engine
pub struct PhysicsManager {
    // Rapier3D simulation
    sim: PhysicsSimulation,

    // Wrappers
    surfaces: Arena<Surface>,
    bodies: Arena<RigidBody>,
    colliders: Arena<Collider>,
}

impl Default for PhysicsManager {
    fn default() -> Self {
        Self {
            sim: PhysicsSimulation::new(),
            surfaces: Default::default(),
            bodies: Default::default(),
            colliders: Default::default(),
        }
    }
}

impl PhysicsManager {
    // Step through the simulation
    pub fn step(&mut self) {
        self.sim.step();
    }
    // Add a surface to the manager
    pub fn add_surface(&mut self, surface: Surface) -> PhysicsID<Surface> {
        PhysicsID::new(self.surfaces.insert(surface))
    }
    // Add a rigidbody to the manager
    pub fn add_rigidbody(&mut self, mut rigidbody: RigidBody) -> PhysicsID<RigidBody> {
        
    }
    // Add a shape collider to the manager
    pub fn add_collider(&mut self, mut collider: Collider, rb_id: PhysicsID<RigidBody>) -> PhysicsID<Collider> {
        // Convert isometry
        let position = collider
            .shape
            .try_get_center()
            .unwrap_or(&veclib::Vector3::ZERO);
        let translation = rapier3d::prelude::Translation::new(position.x, position.y, position.z);
        // Create collider
        let shared_shape = match &collider.shape {
            ColliderType::Cuboid(cuboid) => SharedShape::cuboid(
                cuboid.size.x / 2.0,
                cuboid.size.y / 2.0,
                cuboid.size.z / 2.0,
            ),
            ColliderType::Sphere(sphere) => SharedShape::ball(sphere.radius),
            ColliderType::Mesh(mesh) => panic!(),
        };
        // Get the surface parameters
        let surface = self.surfaces.get(collider.surface.inner).unwrap();

        // Build the collider
        let rapier_collider = ColliderBuilder::new(shared_shape)
            .position(rapier3d::na::Isometry {
                rotation: Rotation::default(),
                translation,
            })
            .restitution(surface.restitution)
            .friction(surface.friction)
            .build();
        // And add it
        let rigidbody = self.bodies.get(rb_id.inner).unwrap();
        // Set the new handle
        collider.handle = self.sim.colliders.insert_with_parent(rapier_collider, rigidbody.handle, &mut self.sim.bodies);
        // Create the ID now
        PhysicsID::new(self.colliders.insert(collider))
    }
}
