use others::{Instance, SmartList};
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU8, Arc},
};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub next_id: usize, 
    pub entities: HashMap<usize, Entity>,
    pub entities_to_delete: HashMap<usize, u8>,
}

impl EntityManager {
    // Get an entity
    pub fn entity(&self, entity_id: usize) -> Option<&Entity> {
        self.entities.get(&entity_id)
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, entity_id: usize) -> Option<&mut Entity> {
        self.entities.get_mut(&entity_id)
    }
    // Add an entity to the manager
    pub fn add_entity(&mut self, mut entity: Entity) -> usize {
        // Set the entity ID
        entity.entity_id = self.next_id;
        self.entities.insert(entity.entity_id, entity);
        self.next_id += 1;
        self.next_id - 1
    }
    // Remove an entity from the manager, and return it's value
    pub fn remove_entity(&mut self, entity_id: usize) -> Option<Entity> {
        self.entities.remove(&entity_id)
    }
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,                             // The name of the entity
    pub system_bitfield: u32,                     // A bitfield for the System IDs that we are part of
    pub entity_id: usize,                         // The ID of the entity
    pub linked_components: HashMap<usize, usize>, // A hash map containing the component ID to global ID of our components
    pub c_bitfield: usize,                        // Our component bitfield
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
            "Entity '{}': {{ ID: '{}', cBitfield: {}, sBitfield: {}, #Components Linked: '{}' }}",
            self.name,
            self.entity_id,
            self.system_bitfield,
            self.c_bitfield,
            self.linked_components.len()
        )
    }
}
