use world::ecs::component::Component;
pub use world::physics::rapier3d::prelude::RigidBodyType;

// RigidBody component
#[derive(Component)]
pub struct RigidBody {
    // Velocity
    pub velocity: veclib::Vector3<f32>,

    // The state of the rigidbody
    pub _type: RigidBodyType,
    pub sleeping: bool,
}
