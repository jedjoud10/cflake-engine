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
    pub fn new<C>(ecs_manager: &ECSManager<C>) -> Self {
        Self {
            index: ecs_manager.entities.get_next_idx_increment() as u16,
        }
    }
}

// A ComponentID that will be used to identify components
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ComponentID {
    pub(crate) entity_id: EntityID,
    pub(crate) cbitfield: Bitfield<u32>,
}
impl ComponentID {
    // Create a new component ID using a component generic and an entity ID
    pub(crate) fn new(entity_id: EntityID, cbitfield: Bitfield<u32>) -> Self {
        Self { entity_id, cbitfield }
    }
}
