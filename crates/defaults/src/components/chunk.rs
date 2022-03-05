use world::{
    ecs::component::Component,
    rendering::{basics::mesh::Mesh, pipeline::Handle},
    terrain::ChunkCoords,
};

// A component that will be added to chunk entities
#[derive(Component)]
pub struct Chunk {
    pub coords: ChunkCoords,
    pub(crate) updated_mesh_id: Handle<Mesh>,
}

impl Chunk {
    // Calculate a chunk's priority using chunk coords and the camera position and direction
    pub fn calculate_priority(coords: ChunkCoords, camera_position: veclib::Vector3<f32>, camera_forward: veclib::Vector3<f32>) -> f32 {
        let position = veclib::Vector3::<f32>::from(coords.position);
        (position - camera_position).normalized().dot(camera_forward)
    }
}
