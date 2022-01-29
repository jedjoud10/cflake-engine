use main::{
    ecs::component::Component,
    terrain::{ChunkCoords, Voxable, VoxelData},
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk<V: Voxable + 'static> {
    pub coords: ChunkCoords,
    pub voxel_data: Option<VoxelData<V>>,
    pub valid_surface: bool,
    pub valid_model: bool,
}

impl<V: Voxable + 'static> Chunk<V> {
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
