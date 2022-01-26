use main::{
    ecs,
    terrain::{ChunkCoords, VoxelData},
};

// A component that will be added to chunk entities
pub struct Chunk {
    pub coords: ChunkCoords,
    pub voxel_data: Option<VoxelData>,
    pub valid_model: bool,
}

// Main traits implemented
ecs::impl_component!(Chunk);

impl Chunk {
    // New
    pub fn new(coords: ChunkCoords) -> Self {
        Self { coords, voxel_data: None, valid_model: false, }
    }
}
