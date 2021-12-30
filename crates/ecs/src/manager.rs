use std::{collections::HashMap, any::Any};
use fnv::{FnvHashMap, FnvHashSet};
use crate::{system::SystemManager, EntityManager, ComponentInternal, identifiers::{ComponentID, EntityID}, Entity, ECSError, Component, EntityError};


// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    entities: Vec<Entity>, // A vector full of entities. Each entity can get invalidated, but never deleted
    free_entities: Vec<u16>, // A list of free entities, we
    components: FnvHashMap<ComponentID, Box<dyn ComponentInternal + Sync + Send>>, // The components that are valid in the world 
}
// Global code for the Entities, Components, and Systems
impl ECSManager {
    /* #region Entities */
    // Get an entity
    pub fn entity(&self, id: EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), id))
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, id: EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), id))
    }
    // Add an entity to the manager
    pub fn add_entity(&mut self, entity: Entity) -> EntityID {
        // If we have any 
        // Create a new EntityID for this entity
        let entity_id = EntityID::new(self.entities.len() as u16);
        // Add the entity
        self.entities.push(entity);
        entity_id
    }
    // Remove an entity from the manager, and return it's value
    pub fn remove_entity(&mut self, id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate an entity
        let entity = self.entity_mut(id)?;
        // Create a default nul Entity 
        let entity = std::mem::replace(entity, Entity::new());
        self.free_entities.push(id.index);
        Ok(entity)
    }
    /* #endregion */
    /* #region Components */
    // Add a specific linked componment to the component manager. Return the said component's ID
    pub fn add_component(&mut self, component: Box<dyn ComponentInternal + Send + Sync + 'static>) -> Result<ComponentID, ECSError> {
        // Create a new Component ID from an Entity ID
        let global_id = self.components.insert(component);
        Ok(global_id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T: ComponentInternal + 'static>(linked_component: &'a dyn ComponentInternal, global_id: usize) -> Result<&T, ECSError> {
        let component_any: &dyn Any = linked_component.as_any();
        let reference = component_any.downcast_ref::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))?;
        Ok(reference)
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T: ComponentInternal + 'static>(boxed_component: &'a mut dyn ComponentInternal, global_id: usize) -> Result<&mut T, ECSError> {
        let component_any: &mut dyn Any = boxed_component.as_any_mut();
        let reference_mut = component_any.downcast_mut::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))?;
        Ok(reference_mut)
    }
    // Get a reference to a specific linked component
    pub fn get_component<'a, T: Component + 'static>(&'a self, global_id: usize) -> Result<&T, ECSError> {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let linked_component = self
            .components
            .get_element(global_id)
            .flatten()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component::<T>(linked_component.as_ref(), global_id)?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn get_component_mut<'a, T: Component + 'static>(&'a mut self, global_id: usize) -> Result<&mut T, ECSError> {
        let linked_component = self
            .components
            .get_element_mut(global_id)
            .flatten()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component_mut::<T>(linked_component.as_mut(), global_id)?;
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn remove_component(&mut self, global_id: usize) -> Result<(), ECSError> {
        // To remove a specific component just set it's component slot to None
        self.components.remove_element(global_id).unwrap();
        Ok(())
    }
    /* #endregion */
}