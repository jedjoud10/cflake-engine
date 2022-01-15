use std::{cell::RefCell, sync::Arc};

use crate::{
    identifiers::EntityID,
    ComponentError, ComponentLinkingGroup, Entity, EntityError, System, ComponentID, EnclosedComponent, linked_components::LinkedComponents, event_handler::EventHandler,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::{shareable::ShareableOrderedVec, simple::OrderedVec};
use worker_threads::ThreadPool;

// The Entity Component System manager that will handle everything ECS related (other than the components)
pub struct ECSManager {
    // A vector full of entities. Each entity can get invalidated, but never deleted
    pub(crate) entities: ShareableOrderedVec<Entity>, 
    // Each system, stored in the order they were created
    systems: Vec<System>,                             
    // The components that are valid in the world
    pub(crate) components_ids: AHashMap<ComponentID, usize>,
    pub(crate) components: OrderedVec<RefCell<EnclosedComponent>>, 
    // The internal ECS thread pool
    pub(crate) thread_pool: ThreadPool<LinkedComponents>,
}

// Global code for the Entities, Components, and Systems
impl ECSManager {
    // Create a new ECS manager
    pub fn new<F: Fn() + Sync + Send + 'static>(start_function: F) -> Self {
        // Start the thread pool
        let thread_pool = ThreadPool::new(8, start_function);
        Self { 
            entities: Default::default(),
            systems: Default::default(),
            components_ids: Default::default(),
            components: Default::default(),
            thread_pool,
        }
    }
    /* #region Entities */
    // Get an entity
    pub fn entity(&self, id: &EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, id: &EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add_entity(&mut self, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup) {
        // Check if the EntityID was not occupied already
        if self.entities.get(id.index as usize).is_some() {
            panic!()
        }
        entity.id = Some(id);
        // Add the entity
        let idx = self.entities.insert(id.index as usize, entity);
        // After doing that, we can safely add the components
        self.add_component_group(id, group).unwrap();
    }
    // Remove an entity from the manager, and return it's value
    // When we remove an entity, we also remove it's components, thus updating the systems
    pub fn remove_entity(&mut self, id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate the entity
        let entity = self.entities.remove(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), id))?;
        // Also remove it's linked components
        for component_id in entity.components.iter() {
            self.remove_component(*component_id).unwrap();
        }
        Ok(entity)
    }
    /* #endregion */
    /* #region Components */
    // Add a component linking group to the manager
    fn add_component_group(&mut self, id: EntityID, group: ComponentLinkingGroup) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in group.linked_components {
            let idx = self.add_component(id, boxed, cbitfield)?;
            let entity = self.entity_mut(&id).unwrap();
            entity.components.push(idx);
        }        
        // Check if the linked entity is valid to be added into the systems
        self.systems.iter_mut().for_each(|system| system.check_add_entity(group.cbitfield, id));
        Ok(())
    }    
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, id: EntityID, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError> {
        // We must make this a RefCell
        let cell = RefCell::new(boxed);
        let idx = self.components.push_shove(cell);
        // Create a new Component ID
        let id = ComponentID::new(cbitfield, idx);
        self.components_ids.insert(id, idx);
        Ok(id)
    }
    // Remove a specified component from the list
    fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        let idx = self.components_ids.remove(&id).ok_or(ComponentError::new("Tried removing component, but it was not present in the ECS manager!".to_string(), id))?;
        self.components.remove(idx);
        Ok(())
    }
    /* #endregion */
    /* #region Systems */
    // Add a system to our current systems
    pub fn add_system(&mut self, system: System) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn systems(&self) -> &[System] {
        self.systems.as_ref()
    }
    // Run the systems in sync, but their component updates is not
    // For now we will run them on the main thread, until I get my thread pool thingy working
    pub fn run_systems<'long: 'short, 'short, RefContext: Clone + 'short>(&'long self, context: &'short RefContext, event_handler: &EventHandler<RefContext>) {
        for system in self.systems.iter() {
            system.run_system(context.clone(), event_handler, self);
        }
    }
    /* #endregion */
}
