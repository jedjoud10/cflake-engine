use crate::EntityID;
use bitfield::Bitfield;
// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
    pub(crate) id: Option<EntityID>,     // This entity's ID
    pub(crate) cbitfield: Bitfield<u32>, // Component Bitfield
}

// ECS time bois
impl Entity {
    // Create a new default entity
    pub fn new() -> Self {
        Self {
            id: None,
            cbitfield: Bitfield::default(),
        }
    }
}
