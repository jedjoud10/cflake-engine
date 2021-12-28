use others::{Instance, SmartList};
use std::{collections::{HashMap, HashSet}, sync::{atomic::AtomicU8, Arc}};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: SmartList<Entity>,
    pub entities_to_delete: HashMap<usize, u8>,
}

impl EntityManager {
    // Get an entity
    pub fn entity(&self, entity_id: usize) -> &Entity {
        self.entities.get_element(entity_id).flatten().unwrap()
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, entity_id: usize) -> &mut Entity {
        self.entities.get_element_mut(entity_id).flatten().unwrap()
    }
    // Add an entity to the manager
    pub fn add_entity(&mut self, mut entity: Entity) -> usize {
        // Set the entity ID
        let next_id = self.entities.get_next_valid_id();
        entity.entity_id = next_id;
        self.entities.add_element(entity)
    }
    // Remove an entity from the manager, and return it's value
    pub fn remove_entity(&mut self, entity_id: usize) -> Entity {
        self.entities.remove_element(entity_id).unwrap()
    }
    // Check if an entity is valid
    pub fn is_entity_valid(&self, entity_id: usize) -> bool {
        self.entities.get_element(entity_id).flatten().is_some() && !self.entities_to_delete.contains_key(&entity_id)
    }
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String, // The name of the entity
    pub system_bitfield: u32, // A bitfield for the System IDs that we are part of
    pub entity_id: usize, // The ID of the entity
    pub linked_components: HashMap<usize, usize>, // A hash map containing the component ID to global ID of our components
    pub c_bitfield: usize, // Our component bitfield
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
