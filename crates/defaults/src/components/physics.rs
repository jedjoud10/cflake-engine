use main::physics::PhysicsObject;

// A physics component
#[derive(Default, Clone)]
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

// Main traits implemented
main::ecs::impl_component!(Physics);
