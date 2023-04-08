use ecs::Component;

// This is what we will add onto the camera to let it dynamically generate chunks
// The chunk viewer must always have a position component attached to it
#[derive(Default, Component)]
pub struct ChunkViewer;

// State of the indirect mesh of each chunk
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    // Only used as an internal state for removing/adding chunks
    Free,

    // Chunk *must* be regenerated, doesn't mean it currently will be though
    Dirty,

    // The chunk is waiting for the compute shader to generate it's mesh
    Pending,

    // The chunk's mesh has been generated successfully
    Generated,
}

// Coordniate system for chunks
pub type ChunkCoords = vek::Vec3<i32>;

// This is a terrain chunk component that will be automatically added
// on entities that are generated from the terrain generator
// Each chunk has a specific state associated with it that represents what stage it's in for terrain generation
#[derive(Component)]
pub struct Chunk {
    pub(crate) allocation: usize,
    pub(crate) local_index: usize,
    pub(crate) global_index: usize,
    pub(crate) ranges: Option<vek::Vec2<u32>>,
    pub(crate) state: ChunkState,
    pub(crate) coords: ChunkCoords,
    pub(crate) priority: f32,
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

    // Force the regeneration of a specific chunk
    pub fn regenerate(&mut self) {
        self.state = ChunkState::Dirty;
    }
}
