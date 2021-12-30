use std::{any::Any};
use fnv::{FnvHashMap, FnvHashSet};
use crate::{identifiers::{ComponentID, EntityID}, Entity, Component, EntityError, ComponentError};


// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    entities: Vec<Entity>, // A vector full of entities. Each entity can get invalidated, but never deleted
    free_entities: Vec<u16>, // A list of free entities, we
    components: FnvHashMap<ComponentID, Box<dyn Component + Sync + Send>>, // The components that are valid in the world 
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
    pub fn add_component<T>(&mut self, entity_id: EntityID, component: T) -> Result<ComponentID, ComponentError>
        where T: Component + Send + Sync + 'static
    {
        // Create a new Component ID from an Entity ID
        let id = ComponentID::new::<T>(entity_id);
        // We must box the component
        let boxed = Box::new(component);
        self.components.insert(id, boxed);
        Ok(id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T>(linked_component: &'a dyn Component, id: ComponentID) -> Result<&T, ComponentError> 
        where T: Component + Send + Sync + 'static
    {
        let component_any: &dyn Any = linked_component.as_any();
        let reference = component_any.downcast_ref::<T>().ok_or_else(|| ComponentError::new("Could not cast component".to_string(), id))?;
        Ok(reference)
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T>(linked_component: &'a mut dyn Component, id: ComponentID) -> Result<&mut T, ComponentError>
        where T: Component + Send + Sync + 'static
    {
        let component_any: &mut dyn Any = linked_component.as_any_mut();
        let reference_mut = component_any.downcast_mut::<T>().ok_or_else(|| ComponentError::new("Could not cast component".to_string(), id))?;
        Ok(reference_mut)
    }
    // Get a reference to a specific linked component
    pub fn get_component<T>(&self, id: ComponentID) -> Result<&T, ComponentError>
        where T: Component + Send + Sync + 'static
    {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let linked_component = self
            .components
            .get(&id)
            .ok_or_else(|| ComponentError::new("Linked component could not be fetched!".to_string(), id))?;
        let component = Self::cast_component::<T>(linked_component.as_ref(), id)?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn get_component_mut<T>(&mut self, id: ComponentID) -> Result<&mut T, ComponentError>
        where T: Component + Send + Sync + 'static
    {
        let linked_component = self
            .components
            .get_mut(&id)
            .ok_or_else(|| ComponentError::new("Linked component could not be fetched!".to_string(), id))?;
        let component = Self::cast_component_mut::<T>(linked_component.as_mut(), id)?;
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        self.components.remove(&id).ok_or(ComponentError::new("Tried removing component, but it was not present in the HashMap!".to_string(), id));
        Ok(())
    }
    /* #endregion */
}