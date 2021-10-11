use ecs::{Component, ComponentID, ComponentInternal};
use terrain::ChunkCoords;

// A component that will be added to well... chunks
#[derive(Default)]
pub struct Chunk {
    pub coords: ChunkCoords,
}

// Main traits implemented
ecs::impl_component!(Chunk);
