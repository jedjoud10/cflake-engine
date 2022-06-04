use world::{
    ecs::Component,
    terrain::{ChunkCoords, PersistentVoxelData, VoxelDataBufferId},
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    // The chunk's coordinates
    pub coords: ChunkCoords,

    // The specific voxel data buffer index that we used to generate our mesh
    pub voxel_data_id: Option<VoxelDataBufferId>,

    // Persistent chunk voxel data that we use for pathfinding and such
    pub persistent: Option<PersistentVoxelData>,
}

impl Chunk {
    // Calculate a chunk's priority using chunk coords and the camera position and direction
    pub fn calculate_priority(coords: ChunkCoords, camera_position: vek::Vec3<f32>, camera_forward: vek::Vec3<f32>) -> f32 {
        let position = coords.position.as_();
        (position - camera_position).normalized().dot(camera_forward)
    }
}
