use super::component::{Component, ComponentID, ComponentManager};
use super::error::ECSError;
use std::collections::HashMap;

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: HashMap<u16, Entity>,
    pub entitites_to_add: Vec<Entity>,
}

impl EntityManager {
    // Add an entity to the entity manager
    pub fn internal_add_entity(&mut self, mut entity: Entity) -> u16 {
        entity.entity_id = self.entities.len() as u16;
        // Add the entity to the world
        let id = entity.entity_id;
        println!("{:?}", entity);
        self.entities.insert(entity.entity_id, entity);
        id
    }
    // Add an entity to the entity manager temporarily, then call the actual add entity function on the world to actually add it
    pub fn add_entity_s(&mut self, mut entity: Entity) -> u16 {
        // Temporarily add it to the entities_to_add vector

        // Get the id of the entity inside the temp vector (Local ID)
        let mut id = self.entitites_to_add.len() as u16;
        // Add that id to the id of the current vector length (Global ID)
        id += self.entities.len() as u16;
        entity.entity_id = id;
        self.entitites_to_add.push(entity);
        id
    }
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(&mut self, entity_id: u16) -> Result<&mut Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            return Ok(self.entities.get_mut(&entity_id).unwrap());
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
    // Get an entity using it's entity id
    pub fn get_entity(&self, entity_id: u16) -> Result<&Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            return Ok(self.entities.get(&entity_id).unwrap());
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
    // Removes an entity from the world
    pub fn remove_entity(&mut self, entity_id: u16) -> Result<Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            let removed_entity = self.entities.remove(&entity_id).unwrap();
            Ok(removed_entity)
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub linked_components: HashMap<u16, u16>,
    pub c_bitfield: u16,
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
    pub fn link_default_component<T: ComponentID + Default + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
    ) -> Result<(), ECSError> {
        // Simple wrapper around the default link component
        self.link_component(component_manager, T::default())
    }
    // Check if we have a component linked
    pub fn is_component_linked(&self, component_id: &u16) -> bool {
        self.linked_components.contains_key(component_id)
    }
    // Link a component to this entity and use the given default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
        default_state: T,
    ) -> Result<(), ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we have the component linked on this entity
        if !self.linked_components.contains_key(&component_id) {
            // The component was not linked yet, link it
            // Add the component and get the global ID and add it to our hashmap
            let global_id = component_manager.add_linked_component::<T>(default_state)?;
            // Add the global ID to our hashmap
            self.linked_components.insert(component_id, global_id);
        } else {
            // The component was already linked
            return Err(ECSError::new(
                format!(
                    "Cannot link component '{}' to entity '{}' because it is already linked!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(
        &mut self,
        component_manager: &mut ComponentManager,
    ) -> Result<(), ECSError> {
        let _name = T::get_component_name();
        let id = component_manager.get_component_id::<T>()?;
        let global_id = self.linked_components.get(&id).unwrap();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;

        // Get the linked components and remove the component from it
        component_manager.id_remove_linked_component(global_id)?;
        return Ok(());
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we even have the component
        if self.is_component_linked(&component_id) {
            let global_id = self.linked_components.get(&component_id).unwrap();
            let final_component = component_manager.id_get_linked_component::<T>(global_id)?;
            Ok(final_component)
        } else {
            return Err(ECSError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
    // Gets a specific component, mutably
    pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a mut ComponentManager,
    ) -> Result<&'a mut T, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we even have the component
        if self.is_component_linked(&component_id) {
            let global_id = self.linked_components.get(&component_id).unwrap();
            let final_component = component_manager.id_get_linked_component_mut::<T>(global_id)?;
            Ok(final_component)
        } else {
            return Err(ECSError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
}
