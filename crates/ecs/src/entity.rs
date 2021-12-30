use bitfield::Bitfield;
use crate::identifiers::EntityID;
// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
    pub id: EntityID, // This entity's ID
    pub cbitfield: Bitfield<u32>, // Component Bitfield
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new() -> Self {
        Self {
            id: EntityID::new(0),
            cbitfield: Bitfield::default(),
        }
    }
}