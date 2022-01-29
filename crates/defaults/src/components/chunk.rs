use main::{
    ecs,
    terrain::{ChunkCoords, VoxelData, Voxable},
};

// A component that will be added to chunk entities
pub struct Chunk<V> {
    pub coords: ChunkCoords,
    pub voxel_data: Option<VoxelData<V>>,
    pub valid_surface: bool,
    pub valid_model: bool,
}

// Main traits implemented
ecs::impl_component!(Chunk);

impl Chunk {
    // New
    pub fn new(coords: ChunkCoords) -> Self {
        Self {
            coords,
            voxel_data: None,
            valid_surface: false,
            valid_model: false,
        }
    }
}
