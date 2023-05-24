use ecs::Component;

// A rigidbody is an object that is affect by external forces and collisions
// It basically represents any physics object in the world scene that 
#[derive(Component)]
pub struct RigidBody {
    // The mass of the rigid body in kG
    mass: f32,
}

impl RigidBody {
    // Create a new RigidBody with a specific mass in kG
    pub fn new(mass: f32) -> Self {
        Self { mass }
    }
}