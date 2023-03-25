use ecs::Component;

// This is what we will add onto the camera to let it dynamically generate chunks
// The chunk viewer must always have a position component attached to it
#[derive(Default, Component)]
pub struct ChunkViewer;

// State of the indirect mesh of each chunk
#[derive(Clone, Copy)]
pub enum ChunkState {
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
    pub(crate) state: ChunkState,
    pub(crate) coords: ChunkCoords,
}

impl Chunk {
    // Get the chunk coordinates
    pub fn coords(&self) -> ChunkCoords {
        self.coords
    }

    // Get the current chunk state
    pub fn state(&self) -> ChunkState {
        self.state
    }
}
