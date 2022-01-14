use crate::{EntityID, EnclosedComponent, ComponentID};
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
