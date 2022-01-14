use crate::{ECSManager, Entity};
use bitfield::Bitfield;
use ordered_vec::shareable::*;
// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID {
    pub index: u16,
}
impl EntityID {
    // Create a new entity ID using a ShareableOrderedVecState of the entities, something that we can get by the Context<ECSManager>
    pub fn new(ecs_manager: &ECSManager) -> Self {
        Self {
            index: ecs_manager.entities.get_next_idx_increment() as u16,
        }
    }
}

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub(crate) cbitfield: Bitfield<u32>,
    pub(crate) idx: usize,
}
impl ComponentID {
    // Create a new component ID
    pub(crate) fn new(cbitfield: Bitfield<u32>, idx: usize) -> Self {
        Self { cbitfield, idx }
    }
}
