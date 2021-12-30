use bitfield::Bitfield;

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID {
    pub index: u16,
}
impl EntityID {
    pub fn new(index: u16) -> Self {
        Self { index }
    }
}

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub entity_id: EntityID,
    pub cbitfield: Bitfield<u32>,
}
impl ComponentID {
    // Create a new component ID using a component generic and an entity ID
    pub fn new(id: EntityID, cbitfield: Bitfield<u32>) -> Self {
        Self { entity_id: id, cbitfield }
    }
}

// Contains some data about the actual system on the worker thread
pub struct SystemThreadData {
    pub join_handle: std::thread::JoinHandle<()>, // A join handle for the system worker thread
    pub cbitfield: Bitfield<u32>,                 // The linked component Bitfield that we made for this system
}

impl SystemThreadData {
    // New
    pub fn new(join_handle: std::thread::JoinHandle<()>, cbitfield: Bitfield<u32>) -> Self {
        Self { join_handle, cbitfield }
    }
}
