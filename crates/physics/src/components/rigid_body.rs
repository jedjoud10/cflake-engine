use ecs::Component;
pub use rapier3d::prelude::RigidBodyType;

// A rigidbody is an object that is affect by external forces and collisions
// It basically represents any physics object in the world scene that 
#[derive(Component)]
pub struct RigidBody {
    pub _type: RigidBodyType,
    pub(crate) interpolated: bool,
    pub(crate) sleeping: bool,
    pub(crate) handle: Option<rapier3d::dynamics::RigidBodyHandle>,
}

impl Clone for RigidBody {
    fn clone(&self) -> Self {
        Self {
            _type: self._type.clone(),
            sleeping: false,
            handle: None,
            interpolated: self.interpolated,
        }
    }
}

impl RigidBody {
    // Create a new RigidBody with a specific mass in kG
    pub fn new(_type: RigidBodyType, interpolated: bool) -> Self {
        Self { 
            _type,
            interpolated,
            handle: None,
            sleeping: false,
        }
    }

    // Check if the RigidBody is currently sleeping
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    // Apply an impulse on the rigid-body
    pub fn apply_impulse(&mut self, impulse: vek::Vec3<f32>) {
    }

    // Apply a torque impulse on the rigid-body
    pub fn apply_torque_impulse(&mut self, impulse: vek::Vec3<f32>) {
    }

    // Apply an impulse at a specific point on the rigid-body
    pub fn apply_impulse_at_point(&mut self, impulse: vek::Vec3<f32>, point: vek::Vec3<f32>) {
    }

    // Add a force to the rigid-body
    pub fn add_force(&mut self, force: vek::Vec3<f32>) {
    }

    // Add a torque force on the rigid-body
    pub fn add_torque(&mut self, torque: vek::Vec3<f32>) {
    }

    // Apply a force at a specific point on the rigid-body
    pub fn add_force_at_point(&mut self, force: vek::Vec3<f32>, point: vek::Vec3<f32>) {
    }
}