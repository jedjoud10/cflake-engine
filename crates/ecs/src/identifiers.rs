use std::{ptr::{null_mut, null}, sync::{atomic::{AtomicPtr, Ordering::Relaxed}, Arc}};

use bitfield::Bitfield;
use others::ExternalID;
// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct IEntityID {
    pub index: u16,
}
impl IEntityID {
    pub fn new(index: u16) -> Self {
        Self {
            index
        }
    }
}

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub(crate) entity_id: IEntityID,
    pub(crate) cbitfield: Bitfield<u32>,
}
impl ComponentID {
    // Create a new component ID using a component generic and an entity ID
    pub(crate) fn new(entity_id: IEntityID, cbitfield: Bitfield<u32>) -> Self {
        Self { entity_id, cbitfield }
    }
}