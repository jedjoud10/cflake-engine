use super::{Component, ComponentID, ComponentManager};
use errors::ECSError;
use others::Instance;
use std::collections::{HashMap, HashSet};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: Vec<Option<Entity>>,

    // Entities to add / remove from systems
    pub entities_to_remove: HashSet<usize>,
    pub entities_to_add: Vec<Entity>,
}

impl EntityManager {
    // Calculate the next valid ID from the actual entities
    pub fn get_next_valid_id(&self) -> usize {
        // Calculate the next valid free ID
        return self
            .entities
            .iter()
            .enumerate()
            .position(|(i, e)| {
                match e {
                    // We found a free spot
                    Some(entity) => false,
                    None => true,
                }
            })
            .unwrap_or(self.entities.len());
    }
    // Add an entity to the entity manager temporarily, then call the actual add entity function on the world to actually add it
    pub fn add_entity_s(&mut self, mut entity: Entity) -> usize {
        // Get the id of the entity inside the temp vector (Local ID)
        let id = self.get_next_valid_id();
        entity.entity_id = id;
        // Update
        if id < self.entities.len() {
            // Turn the none into a valid entity
            self.entities[id as usize] = Some(entity.clone());
        } else {
            // Add this to the entities
            self.entities.push(Some(entity.clone()));
        }
        self.entities_to_add.push(entity);
        id
    }
    // Remove an entity from the entity manager temporarily, then call the actual removal function in the world to actually remove it
    pub fn remove_entity_s(&mut self, entity_id: usize) -> Result<Option<Entity>, ECSError> {
        // If we wish to remove an entity that was already queued for removal, don't do anything
        if self.entities_to_remove.contains(&entity_id) {
            let entity = self.entities.get(entity_id).unwrap().clone();
            return Ok(Some(entity.unwrap()));
        }
        // Ez check first
        if entity_id < self.entities.len() {
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
                let entity = self.entities.get(entity_id).unwrap().clone();
                return Ok(Some(entity.unwrap()));
            }
        } else {
            return Err(ECSError::new_str("Not good"));
        }
    }
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(&mut self, entity_id: usize) -> Result<&mut Entity, ECSError> {
        if entity_id < self.entities.len() {
            let entity = self.entities.get_mut(entity_id).unwrap().as_mut().unwrap();
            return Ok(entity);
        } else {
            return Err(ECSError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id)));
        }
    }
    // Get an entity using it's entity id
    pub fn get_entity(&self, entity_id: usize) -> Result<&Entity, ECSError> {
        if entity_id < self.entities.len() {
            let entity = self
                .entities
                .get(entity_id)
                .unwrap()
                .as_ref()
                .ok_or(ECSError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id)))?;
            return Ok(entity);
        } else {
            return Err(ECSError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id)));
        }
    }
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: usize,
    pub linked_components: HashMap<usize, usize>,
    pub c_bitfield: usize,
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
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default_component<T: Component + 'static>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        // Simple wrapper around the default link component
        self.link_component(component_manager, T::default())
    }
    // Check if we have a component linked
    pub fn is_component_linked(&self, component_id: usize) -> bool {
        self.linked_components.contains_key(&component_id)
    }
    // Link a component to this entity and also link it's default component dependencies if they are not linked yet
    pub fn link_component<T: Component + 'static>(&mut self, component_manager: &mut ComponentManager, default_state: T) -> Result<(), ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we have the component linked on this entity
        if let std::collections::hash_map::Entry::Vacant(e) = self.linked_components.entry(component_id) {
            // The component was not linked yet, link it
            // Add the component and get the global ID and add it to our hashmap
            let global_id = component_manager.add_linked_component::<T>(default_state)?;
            // Add the global ID to our hashmap
            e.insert(global_id);
        } else {
            // The component was already linked
            return Err(ECSError::new(format!(
                "Cannot link component '{}' to entity '{}' because it is already linked!",
                T::get_component_name(),
                self.name
            )));
        }
        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        let global_id = *self.linked_components.get(&id).unwrap();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;

        // Get the linked components and remove the component from it
        component_manager.id_remove_linked_component(global_id)?;
        Ok(())
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
