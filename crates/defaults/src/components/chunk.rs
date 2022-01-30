use main::{
    ecs::component::Component,
    terrain::{ChunkCoords, VoxelData},
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
    pub voxel_data: Option<VoxelData>,
    pub valid_surface: bool,
    pub valid_model: bool,
}

impl Chunk {
    // Create a new chunk from just some chunk coordinates
    pub fn new(coords: ChunkCoords) -> Self {
        Self {
            coords,
            voxel_data: None,
            valid_surface: false,
            valid_model: false,
        }
    }
}
