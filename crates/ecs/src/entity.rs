use crate::{ECSError, LoadState};

use super::{Component, ComponentID, ComponentManager};
use others::{Instance, SmartList};
use std::collections::{HashMap, HashSet};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: SmartList<Entity>,
}

impl EntityManager {
    // Get an entity
    pub fn entity(&self, entity_id: usize) -> &Entity {
        self.entities.get_element(entity_id).as_ref().unwrap().unwrap()
    }
    // Add an entity to the manager
    pub fn add_entity(&mut self, mut entity: Entity) -> usize {
        // Set the entity ID
        let next_id = self.entities.get_next_valid_id();
        entity.entity_id = next_id;
        self.entities.add_element(entity)
    }
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: usize,
    pub linked_components: HashMap<usize, usize>,
    pub c_bitfield: usize,
    pub load_state: LoadState,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }
}

// Each entity is instantiable
impl Instance for Entity {
    fn set_name(&mut self, string: String) {
        self.name = string;
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

// Display
impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity '{}': {{ ID: '{}', cBitfield: {}, #Components Linked: '{}' }}",
            self.name,
            self.entity_id,
            self.c_bitfield,
            self.linked_components.len()
        )
    }
}
