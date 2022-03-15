use world::{ecs::component::Component, terrain::ChunkCoords};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
}

impl Chunk {
    // Calculate a chunk's priority using chunk coords and the camera position and direction
    pub fn calculate_priority(coords: ChunkCoords, camera_position: vek::Vec3<f32>, camera_forward: vek::Vec3<f32>) -> f32 {
        let position = coords.position.as_();
        (position - camera_position).normalized().dot(camera_forward)
    }
}
