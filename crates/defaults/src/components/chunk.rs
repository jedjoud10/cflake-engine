use main::{terrain::{ChunkCoords, VoxelData}, ecs};


// A component that will be added to well... chunks
pub struct Chunk {
    pub coords: ChunkCoords,
    pub voxel_data: Option<Option<VoxelData>>,
}

// Main traits implemented
ecs::impl_component!(Chunk);

impl Chunk {
    // New
    pub fn new(coords: ChunkCoords) -> Self {
        Self { coords, voxel_data: None }
    }
}