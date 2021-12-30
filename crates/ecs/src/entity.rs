use std::collections::HashMap;

use crate::{
    bitfield::{ComponentBitfield, SystemBitfield},
    identifiers::EntityID,
};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub next_id: usize,
    pub entities: HashMap<usize, Entity>,
    pub entities_to_delete: HashMap<usize, u8>,
}
// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
    pub id: EntityID, // This entity's ID
    // Our system bitfield and component bitfield stored in a single variable
    // Component Bitfield is the first 32 bits
    // System Bitfield is the last 32 bits
    pub cbitfield: ComponentBitfield,
    pub sbitfield: SystemBitfield,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new() -> Self {
        Self {
            id: EntityID::new(0),
            cbitfield: ComponentBitfield::default(),
            sbitfield: SystemBitfield::default(),
        }
    }
}
