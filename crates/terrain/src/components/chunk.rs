use ecs::Component;
use math::Node;

// This is what we will add onto the camera to let it dynamically generate chunks
// The chunk viewer must always have a position component attached to it
#[derive(Default, Component)]
pub struct ChunkViewer;

// State of the indirect mesh of each chunk
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkState {
    // Only used as an internal state for removing/adding chunks
    Free,

    // Chunk *must* be regenerated, doesn't mean it currently will be though
    Dirty,

    // The chunk is waiting for the compute shader to generate it's mesh
    Pending,

    // The chunk is waiting for the readback
    PendingReadbackStart,

    // God damnit I fucking hate this shit
    PendingReadbackData,

    // The chunk's mesh has been generated successfully
    Generated { empty: bool, readback: Option<bool>, },

    // The chunk needs to be removed
    PendingRemoval,
}

impl ChunkState {
    pub(crate) fn finished(&self) -> bool {
        match self {
            ChunkState::Generated { .. } | ChunkState::Free => true,
            _ => false,
        }
    }

    pub(crate) fn pending(&self) -> bool {
        match self {
            ChunkState::Dirty
            | ChunkState::Pending
            | ChunkState::PendingReadbackStart
            | ChunkState::PendingReadbackData => true,
            _ => false,
        }
    }
}

// This is a terrain chunk component that will be automatically added
// on entities that are generated from the terrain generator
// Each chunk has a specific state associated with it that represents what stage it's in for terrain generation
#[derive(Component, Clone, Copy, Debug)]
pub struct Chunk {
    pub(crate) allocation: usize,
    pub(crate) local_index: usize,
    pub(crate) ranges: Option<vek::Vec2<u32>>,
    pub(crate) state: ChunkState,
    pub(crate) node: Option<Node>,
    pub(crate) generation_priority: f32,
    pub(crate) readback_priority: Option<f32>,
}

impl Chunk {
    // Corresponding octree node for this chunk
    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    // Get the current chunk state
    pub fn state(&self) -> ChunkState {
        self.state
    }

    // Force the regeneration of a specific chunk by setting it's state to dirty
    pub fn regenerate(&mut self) {
        if let ChunkState::Generated { empty, .. } = self.state {
            self.state = ChunkState::Dirty
        } 
    }

    // Get the allocation used by this chunk
    pub fn allocation(&self) -> usize {
        self.allocation
    }

    // Get the range used by this chunk
    pub fn ranges(&self) -> Option<vek::Vec2<u32>> {
        self.ranges
    }
}
