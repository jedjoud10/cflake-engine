use main::physics::PhysicsObject;
use main::ecs::component::Component;

// A physics component
#[derive(Default, Clone, Component)]
pub struct Physics {
    pub(crate) object: PhysicsObject,
}

// Influence the internal physics object through here
impl Physics {
    // Update the physic object's velocity
    pub fn set_velocity(&mut self, vel: veclib::Vector3<f32>) {
        self.object.set_velocity(vel);
    }
}