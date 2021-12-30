use crate::{
    identifiers::{ComponentID, EntityID},
    Component, ComponentError, Entity, EntityError, SystemThreadData,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use std::any::Any;

// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    entities: Vec<Entity>,                                                 // A vector full of entities. Each entity can get invalidated, but never deleted
    pending_removal_entities: AHashMap<EntityID, u8>,                    // A hashmap of the entities that we must remove, eventually
    components: AHashMap<ComponentID, Box<dyn Component + Sync + Send>>, // The components that are valid in the world
    systems: Vec<SystemThreadData>,                                        // Each system, stored in the order they were created
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
    pub fn add_entity(&mut self, mut entity: Entity) -> EntityID {
        // Create a new EntityID for this entity
        let entity_id = EntityID::new(self.entities.len() as u16);
        println!("Created entity with ID: {:?}", entity.id);
        entity.id = entity_id;
        // Add the entity
        self.entities.push(entity);
        entity_id
    }
    // Store an entity for removal, and wait until it's specified Linked Systems Counter reaches 0
    pub fn set_pending_removal_entity(&mut self, id: EntityID, starting_count: u8) {
        self.pending_removal_entities.insert(id, starting_count);
    }
    // Decrement the Linked Systems Counter for a specified pending removal entity
    pub fn decrement_removal_counter(&mut self, id: EntityID) -> Option<Result<Entity, EntityError>> {
        let count = self.pending_removal_entities.get_mut(&id).unwrap();
        *count -= 1;
        if *count == 0 {
            // The counter has reached 0, we must actually remove the entity
            let removed_entity = self.remove_entity(id);
            // And also remove it's components
            self.components.retain(|key, _| {
                // Check if the key contains our entity ID
                key.entity_id != id
            });
            Some(removed_entity)
        } else { None }
    }
    // Remove an entity from the manager, and return it's value
    fn remove_entity(&mut self, id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate an entity
        let entity = self.entity_mut(id)?;
        // Create a default nul Entity
        let entity = std::mem::replace(entity, Entity::new());
        Ok(entity)
    }
    /* #endregion */
    /* #region Components */
    // Add a specific linked componment to the component manager. Return the said component's ID
    pub fn add_component(&mut self, id: EntityID, boxed: Box<dyn Component + Send + Sync>, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError>
    {
        // Create a new Component ID from an Entity ID
        let id = ComponentID::new(id, cbitfield);
        println!("Created component with ID: {:?}", id);
        self.components.insert(id, boxed);
        Ok(id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T>(linked_component: &'a dyn Component, id: ComponentID) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let component_any: &dyn Any = linked_component.as_any();
        let reference = component_any
            .downcast_ref::<T>()
            .ok_or_else(|| ComponentError::new("Could not cast component".to_string(), id))?;
        Ok(reference)
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T>(linked_component: &'a mut dyn Component, id: ComponentID) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let component_any: &mut dyn Any = linked_component.as_any_mut();
        let reference_mut = component_any
            .downcast_mut::<T>()
            .ok_or_else(|| ComponentError::new("Could not cast component".to_string(), id))?;
        Ok(reference_mut)
    }
    // Get a reference to a specific linked component
    pub fn component<T>(&self, id: ComponentID) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
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
    pub fn component_mut<T>(&mut self, id: ComponentID) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync + 'static,
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
        self.components
            .remove(&id)
            .ok_or(ComponentError::new("Tried removing component, but it was not present in the HashMap!".to_string(), id))?;
        Ok(())
    }
    /* #endregion */
    /* #region Systems */
    // Add a system to our current systems
    pub fn add_system(&mut self, system: SystemThreadData) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn systems(&self) -> &[SystemThreadData] {
        self.systems.as_ref()
    }    
    // Get a mutable reference to the ecsmanager's systems.
    pub fn systems_mut(&mut self) -> &mut Vec<SystemThreadData> {
        &mut self.systems
    }
    /* #endregion */

}
