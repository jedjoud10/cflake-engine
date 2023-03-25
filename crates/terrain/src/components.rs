use ecs::Component;

// State of the indirect mesh of each chunk 
#[derive(Component)]
pub enum ChunkState {
    // The chunk entity was just added into the world
    Added,

    // The chunk is waiting for the compute shader to generate it's mesh
    Pending,

    // The chunk's mesh is generated successfully
    Generated,
}

// Coordniate system for chunks
pub type ChunkCoords = vek::Vec3<i32>;

// This is a terrain chunk component that will be automatically added
// on entities that are generated from the terrain generator
// Each chunk has a specific state associated with it that represents what stage it's in for terrain generation
#[derive(Component)]
pub struct Chunk {
    pub state: ChunkState,
    pub coords: ChunkCoords,
}
