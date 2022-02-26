use rapier3d::math::{Rotation, Vector};
use rapier3d::na::{Quaternion, Unit, UnitQuaternion};
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyType, SharedShape};

use crate::colliders::{Collider, ColliderType};
use crate::identifier::{PhysicsID, PhysicsIDType};
use crate::rigidbody::*;
use crate::simulation::PhysicsSimulation;
use crate::surface::Surface;

// Physics manager that contains everything related to the Rapier3D physics engine
pub struct PhysicsManager {
    // Rapier3D simulation
    sim: PhysicsSimulation,

    // Surfaces
    surfaces: Vec<Surface>,
}

impl Default for PhysicsManager {
    fn default() -> Self {
        Self {
            sim: PhysicsSimulation::new(),
            surfaces: Default::default(),
        }
    }
}

impl PhysicsManager {
    // Add a surface to the manager
    pub fn add_surface(&mut self, surface: Surface) -> PhysicsID<Surface> {
        let id = self.surfaces.len();
        self.surfaces.push(surface);
        PhysicsID::new(PhysicsIDType::Surface(id))
    }
    // Add a rigidbody to the manager
    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) -> PhysicsID<RigidBody> {
        // Convert our rigidbody to a Rapier3D rigidbody
        let _type = match rigidbody.state {
            RigidBodyState::Static => RigidBodyType::Static,
            RigidBodyState::Dynamic => RigidBodyType::Dynamic,
        };
        // Convert isometry
        let translation = rapier3d::prelude::Translation::new(
            rigidbody.position.x,
            rigidbody.position.y,
            rigidbody.position.z,
        );
        let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
            rigidbody.rotation[0],
            rigidbody.rotation[1],
            rigidbody.rotation[2],
            rigidbody.rotation[3],
        ));
        // Create rigidbody
        let rigidbody = RigidBodyBuilder::new(_type)
            .position(rapier3d::na::Isometry {
                rotation,
                translation,
            })
            .build();
        // And add it
        let wrapped = self.sim.bodies.insert(rigidbody);

        // Create the ID now
        PhysicsID::new(PhysicsIDType::RigidBody(wrapped))
    }
    // Add a shape collider to the manager
    pub fn add_collider(&mut self, collider: Collider, rigidbody: PhysicsID<RigidBody>) -> PhysicsID<Collider> {
        // Convert isometry
        let position = collider
            .shape
            .try_get_center()
            .unwrap_or(&veclib::Vector3::ZERO);
        let translation = rapier3d::prelude::Translation::new(position.x, position.y, position.z);
        // Create collider
        let shared_shape = match collider.shape {
            ColliderType::Cuboid(cuboid) => SharedShape::cuboid(
                cuboid.size.x / 2.0,
                cuboid.size.y / 2.0,
                cuboid.size.z / 2.0,
            ),
            ColliderType::Sphere(sphere) => SharedShape::ball(sphere.radius),
            ColliderType::Mesh(mesh) => panic!(),
        };
        // Get the surface parameters
        let idx = *collider.surface.inner.as_surface().unwrap();
        let surface: &Surface = self.surfaces.get(idx).unwrap();

        // Build the collider
        let collider = ColliderBuilder::new(shared_shape)
            .position(rapier3d::na::Isometry {
                rotation: Rotation::default(),
                translation,
            })
            .build();
        // And add it
        let wrapped = self.sim.colliders.insert_with_parent(collider, *rigidbody.inner.as_rigid_body().unwrap(), &mut self.sim.bodies);
        // Create the ID now
        PhysicsID::new(PhysicsIDType::Collider(wrapped))
    }
}
