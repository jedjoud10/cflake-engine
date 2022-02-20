use main::{
    ecs::component::Component, terrain::ChunkCoords,
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
}
