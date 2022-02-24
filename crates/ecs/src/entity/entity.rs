use crate::{component::Component, ECSManager};
use ahash::AHashMap;
use bitfield::Bitfield;
// A simple entity in the world
pub struct Entity {
    // This entity's ID
    pub(crate) id: Option<EntityID>,

    // Component Bitfield
    pub(crate) cbitfield: Bitfield<u32>,

    // Our stored components
    pub(crate) components: AHashMap<Bitfield<u32>, u64>,
}

// ECS time bois
impl Default for Entity {
    // Create a new default entity
    fn default() -> Self {
        Self {
            id: None,
            cbitfield: Bitfield::default(),
            components: AHashMap::new(),
        }
    }
}

impl Entity {
    // Check if we have a component linked onto this entity
    pub fn is_component_linked<T: Component + 'static>(&self) -> bool {
        // Get the cbitfield of the component
        let cbitfield = crate::component::registry::get_component_bitfield::<T>();
        self.cbitfield.contains(&cbitfield)
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
        Self(ecs_manager.entities.get_next_id_increment())
    }
}
