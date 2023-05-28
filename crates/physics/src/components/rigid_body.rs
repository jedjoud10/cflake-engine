use ecs::Component;
pub use rapier3d::prelude::RigidBodyType;

// A rigidbody is an object that is affect by external forces and collisions
// It basically represents any physics object in the world scene that 
#[derive(Component)]
pub struct RigidBody {
    pub _type: RigidBodyType,
    pub(crate) interpolated: bool,
    pub(crate) handle: Option<rapier3d::dynamics::RigidBodyHandle>,
}

impl Clone for RigidBody {
    fn clone(&self) -> Self {
        Self {
            _type: self._type.clone(),
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
            handle: None
        }
    }
}