// Some chunk component
use ecs::{Component, ComponentID, ComponentInternal};
#[derive(Default)]
pub struct Chunk {
    pub chunk_coords: veclib::Vector3<i64>,
}

impl Chunk {}

// Main traits implemented
ecs::impl_component!(Chunk);
