use ecs::{Component, ComponentID, ComponentInternal};
use physics::PhysicsObject;

// A physics component
#[derive(Default, Clone)]
pub struct Physics {
    pub object: PhysicsObject,
}

// Main traits implemented
ecs::impl_component!(Physics);
