use ecs::Component;
use math::Node;

// This is what we will add onto the camera to let it dynamically generate chunks
// The chunk viewer must always have a position component attached to it
#[derive(Default, Component)]
pub struct ChunkViewer;

// State of the mesh that we are reading back to the CPU for collisions
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeshReadbackState {
    // The mesh data transfer has yet to begin
    PendingReadbackStart,

    // The mesh data transfer begun, we are waiting for results
    PendingReadbackData,

    // Mesh was completed readback to the CPU
    Complete,
}

// State of the indirect mesh of each chunk
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkState {
    // Only used as an internal state for removing/adding chunks
    Free,

    // Chunk *must* be regenerated, doesn't mean it currently will be though
    Dirty,

    // The chunk is waiting for the compute shader to generate it's mesh
    Pending,

    // The chunk is waiting for the readback to begin
    PendingReadbackStart,

    // The chunk is waiting for the readback to complete
    PendingReadbackData,

    // The chunk's mesh has been generated successfully
    Generated { empty: bool },

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
    // Chunk memory settings and its index within our memory
    pub(crate) allocation: usize,
    pub(crate) local_index: usize,
    pub(crate) ranges: Option<vek::Vec2<u32>>,

    // Chunk states
    pub(crate) state: ChunkState,
    pub(crate) node: Option<Node>,

    // Higher values mean that the chunk must generate before other chunks
    pub(crate) generation_priority: f32,
    pub(crate) collider: bool,

    // If this is Some, then the chunk will generate collisions with the readback mesh
    pub(crate) mesh_readback_state: Option<MeshReadbackState>,
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
        if let ChunkState::Generated { .. } = self.state {
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
