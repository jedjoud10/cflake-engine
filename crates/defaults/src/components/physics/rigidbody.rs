use world::ecs::component::Component;
pub use rapier3d::prelude::{RigidBodyHandle, RigidBodyType};
// RigidBody component
#[derive(Component)]
pub struct RigidBody {
    // Handle
    pub(crate) handle: RigidBodyHandle,

    // Velocity
    pub velocity: veclib::Vector3<f32>,
    pub angular_velocity: veclib::Vector3<f32>,

    // The state of the rigidbody
    pub _type: RigidBodyType,
    pub sleeping: bool,
}

impl RigidBody {
    // Create a new rigidbody
    pub fn new(_type: RigidBodyType) -> Self {
        Self {
            handle: RigidBodyHandle::invalid(),
            velocity: veclib::Vector3::ZERO,
            angular_velocity: veclib::Vector3::ZERO,
            _type,
            sleeping: false,
        }
    }
}
