use main::{
    ecs::component::Component,
    rendering::{basics::model::Model, object::ObjectID},
    terrain::{ChunkCoords, VoxelData},
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
    pub voxel_data: Option<VoxelData>,
    pub generated_voxel_data: bool,
    pub buffered_model: Option<ObjectID<Model>>,
    pub added_renderer: bool,
}

impl Chunk {
    // Create a new chunk from just some chunk coordinates
    pub fn new(coords: ChunkCoords) -> Self {
        Self {
            coords,
            voxel_data: None,
            generated_voxel_data: false,
            buffered_model: None,
            added_renderer: false,
        }
    }
}
