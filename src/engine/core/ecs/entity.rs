
use std::{collections::HashMap};
use super::component::{Component, ComponentID, ComponentManager, LinkedComponents};
use super::error::ECSError;
// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub lc_id: u16,
    pub generate_lc_id: bool,
    pub c_bitfield: u16,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            generate_lc_id: true,
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
    // Link a component to this entity and use the given default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
        default_state: T,
    ) -> Result<(), ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();

        // Get the linked entity component from the component manager
        let linked_entity_component: &mut LinkedComponents;

        // Generate a new LinkedComponent ID
        if self.generate_lc_id {
            self.lc_id = component_manager.linked_entity_components.len() as u16;
            self.generate_lc_id = false;
        }

        if component_manager.linked_entity_components.contains_key(&self.lc_id) {
            // It already exists, so just use that
            linked_entity_component = component_manager.get_linked_components_mut(self.lc_id)?;
        } else {
            linked_entity_component = component_manager.add_linked_components(self.lc_id, LinkedComponents::default())?;
        }
        // Check if we have the component linked on this entity
        if linked_entity_component.components.contains_key(&component_id) {
            return Err(ECSError::new(
                format!(
                    "Cannot link component '{}' to entity '{}' because it is already linked!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }

        // Add the component inside the component manager
        linked_entity_component.id_add_component::<T>(default_state, component_id);

        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        let _name = T::get_component_name();
        let id = component_manager.get_component_id::<T>()?;
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;
        
        // Get the linked components and remove the component from it
        let linked_components = component_manager.get_linked_components_mut(self.entity_id)?;
        linked_components.remove_component(&id);
        return Ok(());
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();        
        let lc = component_manager.get_linked_components(self.entity_id)?;
        // Check if we even have the component
        if lc.contains_component(&component_id) {
            let final_component = lc.id_get_component::<T>(&component_id)?;
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
        let lc = component_manager.get_linked_components_mut(self.entity_id)?;
        // Check if we even have the component
        if lc.contains_component(&component_id) {
            let final_component = lc.id_get_component_mut::<T>(&component_id)?;
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
		println!("{:?}", entity);
        self.entitites_to_add.push(entity);
        id
    }
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(
        &mut self,
        entity_id: u16,
    ) -> Result<&mut Entity, ECSError> {
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
            let removed_entity = self
                .entities
                .remove(&entity_id)
				.unwrap();
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
