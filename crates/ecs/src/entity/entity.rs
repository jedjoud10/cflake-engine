use crate::{
    component::{ComponentID, EnclosedComponent},
    ECSManager,
};
use ahash::AHashMap;
use bitfield::Bitfield;
// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
    // This entity's ID
    pub(crate) id: Option<EntityID>,

    // Component Bitfield
    pub(crate) cbitfield: Bitfield<u32>,

    // Our stored components
    pub(crate) components: AHashMap<Bitfield<u32>, u64>,
}

unsafe impl Sync for Entity {}
unsafe impl Send for Entity {}

// ECS time bois
impl Entity {
    // Create a new default entity
    pub fn new() -> Self {
        Self {
            id: None,
            cbitfield: Bitfield::default(),
            components: AHashMap::new(),
        }
    }
}

// An EntityID that will be used to identify entities
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct EntityID(pub u64);

impl std::fmt::Display for EntityID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl EntityID {
    // Create a new entity ID using a ShareableOrderedVecState of the entities, something that we can get by the Context<ECSManager>
    pub fn new<Context>(ecs_manager: &ECSManager<Context>) -> Self {
        Self {
            0: ecs_manager.entities.get_next_id_increment(),
        }
    }
}
