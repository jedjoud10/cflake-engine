use main::{ecs::component::Component, terrain::ChunkCoords, rendering::{object::ObjectID, basics::model::Model}};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
    // The ID of the terrain model for this chunk
    pub(crate) updated_model_id: Option<ObjectID<Model>>,
}
