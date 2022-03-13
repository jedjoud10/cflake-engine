use rapier3d::prelude::{RigidBodyForces, RigidBodyMassProps};
use world::ecs::component::Component;
pub use rapier3d::prelude::{RigidBodyHandle, RigidBodyType};

use crate::systems::physics_system::{vec3_to_vector, vector_to_vec3};
// RigidBody component
#[derive(Component)]
pub struct RigidBody {
    // Handle
    pub(crate) handle: RigidBodyHandle,

    // Velocity
    pub velocity: vek::Vec3<f32>,
    pub angular_velocity: vek::Vec3<f32>,

    // Forces 
    pub force: vek::Vec3<f32>,
    pub torque: vek::Vec3<f32>,
    pub(crate) forces: RigidBodyForces,

    // The state of the rigidbody
    pub _type: RigidBodyType,
    pub sleeping: bool,
}

impl RigidBody {
    // Create a new rigidbody
    pub fn new(_type: RigidBodyType) -> Self {
        Self {
            handle: RigidBodyHandle::invalid(),
            velocity: vek::Vec3::zero(),
            angular_velocity: vek::Vec3::zero(),
            force: vek::Vec3::zero(),
            torque: vek::Vec3::zero(),
            forces: RigidBodyForces::default(),
            _type,
            sleeping: false,
        }
    }
    // Apply a force on this rigidbody
    pub fn apply_force(&mut self, force: vek::Vec3<f32>) {
        self.forces.force += vec3_to_vector(force);
        self.force = vector_to_vec3(self.forces.force);
        self.torque = vector_to_vec3(self.forces.torque);
    }
}
