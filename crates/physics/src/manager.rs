use rapier3d::math::Vector;
use rapier3d::prelude::{RigidBodyBuilder, RigidBodyType};

use crate::state::PhysicsSimulation;
use crate::rigidbody::*;

// Physics manager that contains everything related to the rapier physics engine
pub struct Manager {
    sim: PhysicsSimulation,
}

impl Manager {
    // Add a rigidbody to the manager
    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) {
        // Convert our rigidbody to a Rapier3D rigidbody
        let _type = match rigidbody.state {
            RigidBodyState::Static => RigidBodyType::Static,
            RigidBodyState::Dynamic => RigidBodyType::Dynamic,
        };
        // Convert position
        let translation = rapier3d::prelude::Vector::new(rigidbody.position.x, rigidbody.position.y, rigidbody.position.z);
        let rigidbody = RigidBodyBuilder::new(_type)
            .translation(translation)
            .build();
        self.sim.bodies.insert(rigidbody);
    }
}