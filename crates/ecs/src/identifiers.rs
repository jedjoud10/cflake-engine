use std::{ptr::{null_mut, null}, sync::{atomic::{AtomicPtr, Ordering::Relaxed}, Arc}};

use bitfield::Bitfield;
use crate::Entity;


// An external pointer to EntityID, that we will create on some arbitrary threads and that we will pass to the main thread
// This entity ID actually points to a pointer to the actual entity ID, so we can multithread this safely 
// Nobody will mutate the pointer while it is being read, so we are fine
#[derive(Clone, Debug)]
pub struct EntityID {
    ptr: Arc<AtomicPtr<IEntityID>>,
}

impl EntityID {
    // Create a null pointer
    pub fn new() -> Self {
        Self {
            ptr: Arc::new(AtomicPtr::new(null_mut()))
        }
    }
}

impl others::ExternalID<IEntityID> for EntityID {
    fn ptr(&self) -> &Arc<AtomicPtr<IEntityID>> {
        &self.ptr
    }
}

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct IEntityID {
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