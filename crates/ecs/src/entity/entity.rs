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
}

// ECS time bois
impl Entity {
    // Create a new default entity
    pub fn new() -> Self {
        Self {
            id: None,
            cbitfield: Bitfield::default(),
            components: Vec::new(),
        }
    }
}

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID {
    pub index: u16,
}
impl EntityID {
    // Create a new entity ID using a ShareableOrderedVecState of the entities, something that we can get by the Context<ECSManager>
    pub fn new<Context>(ecs_manager: &ECSManager<Context>) -> Self {
        Self {
            index: ecs_manager.entities.get_next_idx_increment() as u16,
        }
    }
}
