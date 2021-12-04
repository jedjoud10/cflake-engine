use crate::{ECSError, LoadState};

use super::{Component, ComponentID, ComponentManager};
use others::{Instance, SmartList};
use std::collections::{HashMap, HashSet};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: SmartList<Entity>,

    // Entities to add / remove from systems
    pub entities_to_remove: HashSet<usize>,
    pub entities_to_add: Vec<Entity>,
}

impl EntityManager {
    // Add an entity to the entity manager temporarily, then call the actual add entity function on the world to actually add it
    pub fn add_entity_s(&mut self, mut entity: Entity) -> usize {
        // Get the id of the entity inside the temp vector (Local ID)
        entity.entity_id = self.entities.get_next_valid_id();
        let id = self.entities.add_element(entity.clone());
        self.entities_to_add.push(entity);
        id
    }
    // Remove an entity from the entity manager temporarily, then call the actual removal function in the world to actually remove it
    pub fn remove_entity_s(&mut self, entity_id: usize) -> Result<Option<Entity>, ECSError> {
        // If we wish to remove an entity that was already queued for removal, don't do anything
        if self.entities_to_remove.contains(&entity_id) {
            let entity = self.entities.get_element(entity_id).unwrap().cloned();
            return Ok(Some(entity.unwrap()));
        }
        // Ez check first
        if entity_id < self.entities.elements.len() {
            // Check if we can cancel out this entity
            if self.entities_to_add.iter().any(|x| x.entity_id == entity_id) {
                // We have the entity in the entities_to_add vector, so we can cancel it out
                self.entities_to_remove.remove(&entity_id);
                let pos = self.entities_to_add.iter().position(|x| x.entity_id == entity_id).unwrap();
                self.entities_to_add.remove(pos);
                return Ok(None);
            } else {
                // Can't cancel it out, so just add it to the removed vector
                self.entities_to_remove.insert(entity_id);
                let entity = self.entities.get_element(entity_id).unwrap().cloned();
                return Ok(Some(entity.unwrap()));
            }
        } else {
            return Err(ECSError::new_str("Not good"));
        }
    }
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(&mut self, entity_id: usize) -> Result<&mut Entity, ECSError> {
        let entity = self
            .entities
            .get_element_mut(entity_id)
            .ok_or(ECSError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id)))?
            .unwrap();
        Ok(entity)
    }
    // Get an entity using it's entity id
    pub fn get_entity(&self, entity_id: usize) -> Result<&Entity, ECSError> {
        let entity = self
            .entities
            .get_element(entity_id)
            .ok_or(ECSError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id)))?
            .unwrap();
        Ok(entity)
    }
}

// A simple entity in the world
#[derive(Clone, Default)]
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
    
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(&self, component_manager: &'a ComponentManager) -> Result<&'a T, ECSError> {
        let component_id = component_manager.get_component_id::<T>()?;
        // Check if we even have the component
        if self.is_component_linked(component_id) {
            let global_id = self.linked_components.get(&component_id).unwrap();
            let final_component = component_manager.id_get_linked_component::<T>(*global_id)?;
            Ok(final_component)
        } else {
            return Err(ECSError::new(format!("Component '{}' does not exist on Entity '{}'!", T::get_component_name(), self.name)));
        }
    }
    // Gets a specific component, mutably
    pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(&self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
        let component_id = component_manager.get_component_id::<T>()?;
        // Check if we even have the component
        if self.is_component_linked(component_id) {
            let global_id = self.linked_components.get(&component_id).unwrap();
            let final_component = component_manager.id_get_linked_component_mut::<T>(*global_id)?;
            Ok(final_component)
        } else {
            return Err(ECSError::new(format!("Component '{}' does not exist on Entity '{}'!", T::get_component_name(), self.name)));
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
