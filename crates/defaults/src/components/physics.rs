use physics::PhysicsObject;
use ecs::{Component, ComponentID, ComponentInternal};

// A physics component
#[derive(Default)]
pub struct Physics {
    pub object: PhysicsObject,
}

// Main traits implemented
ecs::impl_component!(Physics);