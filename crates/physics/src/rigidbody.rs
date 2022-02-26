// A simple rigidbody that might be affected by physics
pub struct RigidBody {
    // Initial values
    pub position: veclib::Vector3<f32>,
    pub velocity: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,

    // Physics
    pub state: RigidBodyState,
} 

// The state of the rigidbody, either Static or Dynamic
pub enum RigidBodyState {
    Static, Dynamic
}