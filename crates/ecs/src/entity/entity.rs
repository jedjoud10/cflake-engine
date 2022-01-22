use crate::{component::ComponentID, ECSManager};
use bitfield::Bitfield;
// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
    // This entity's ID
    pub(crate) id: Option<EntityID>,

    // Component Bitfield
    pub(crate) cbitfield: Bitfield<u32>,

    // Our stored components
    pub(crate) components: Vec<ComponentID>,

    // The current entity state
    pub(crate) state: EntityState,
}

// ECS time bois
impl Entity {
    // Create a new default entity
    pub fn new() -> Self {
        Self {
            id: None,
            cbitfield: Bitfield::default(),
            components: Vec::new(),
            state: EntityState::Valid,
        }
    }
    // Turns an &Entity into an Option<&Entity>, and returns None if the current EntityState is set to Removed
    pub fn validity(&self) -> Option<&Entity> {
        if let EntityState::Valid = self.state {
            Some(self)
        } else {
            None
        }
    }
    // Turns an &Entity into an Option<&Entity>, and returns None if the current EntityState is set to Removed
    pub fn validity_mut(&mut self) -> Option<&mut Entity> {
        if let EntityState::Valid = self.state {
            Some(self)
        } else {
            None
        }
    }
}

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID {
    pub id: u64,
}
impl EntityID {
    // Create a new entity ID using a ShareableOrderedVecState of the entities, something that we can get by the Context<ECSManager>
    pub fn new<Context>(ecs_manager: &ECSManager<Context>) -> Self {
        Self {
            id: ecs_manager.entities.get_next_id_increment(),
        }
    }
}

// An entity state that defines how the entity is doing
#[derive(Clone)]
pub enum EntityState {
    // The entity is valid and it exists
    Valid,

    // The entity is going to be removed next frame, so we cannot do anything with it anymore
    Removed,
}
