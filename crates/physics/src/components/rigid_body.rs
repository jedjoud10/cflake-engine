use ecs::Component;
pub use rapier3d::prelude::RigidBodyType;

// A rigidbody is an object that is affect by external forces and collisions
// It basically represents any physics object in the world scene that 
#[derive(Component)]
pub struct RigidBody {
    pub _type: RigidBodyType,
    pub(crate) handle: Option<rapier3d::dynamics::RigidBodyHandle>,
}

impl Clone for RigidBody {
    fn clone(&self) -> Self {
        Self { _type: self._type.clone(), handle: None }
    }
}

impl RigidBody {
    // Create a new RigidBody with a specific mass in kG
    pub fn new(_type: RigidBodyType) -> Self {
        Self { 
            _type,
            handle: None
        }
    }
}