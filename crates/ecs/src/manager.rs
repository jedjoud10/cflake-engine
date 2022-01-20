use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::{shareable::ShareableOrderedVec, simple::OrderedVec};
use std::{cell::RefCell, sync::Arc};
use worker_threads::ThreadPool;

use crate::{
    component::{ComponentID, EnclosedComponent, LinkedComponents},
    entity::{ComponentLinkingGroup, Entity, EntityID, ComponentUnlinkGroup},
    system::{EventHandler, System, SystemBuilder},
    utils::{ComponentError, EntityError},
};

// The Entity Component System manager that will handle everything ECS related (other than the components)
pub struct ECSManager<Context> {
    // A vector full of entities. Each entity can get invalidated, but never deleted
    pub(crate) entities: ShareableOrderedVec<Entity>,
    // Each system, stored in the order they were created
    systems: Vec<System>,
    // The components that are valid in the world
    pub(crate) components_ids: AHashMap<ComponentID, u64>,
    pub(crate) components: OrderedVec<RefCell<EnclosedComponent>>,
    // The internal ECS thread pool
    pub(crate) thread_pool: ThreadPool<LinkedComponents>,
    // Our internal event handler
    pub(crate) event_handler: EventHandler<Context>,
}

// Global code for the Entities, Components, and Systems
impl<Context> ECSManager<Context> {
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
            event_handler: Default::default(),
        }
    }
    /* #region Entities */
    // Get an entity
    pub fn entity(&self, id: &EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.id).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, id: &EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.id).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub(crate) fn add_entity(&mut self, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup) {
        // Check if the EntityID was not occupied already
        if self.entities.get(id.id).is_some() {
            panic!()
        }
        entity.id = Some(id);
        // Add the entity
        let _idx = self.entities.insert(id.id, entity);
        // After doing that, we can safely add the components
        self.link_components(id, group).unwrap();
    }
    // Remove an entity from the manager, and return it's value
    // When we remove an entity, we also remove it's components, thus updating the systems
    pub(crate) fn remove_entity(&mut self, id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate the entity
        let entity = self.entities.remove(id.id).ok_or(EntityError::new("Could not find entity!".to_string(), id))?;
        // Also remove it's linked components
        for component_id in entity.components.iter() {
            self.remove_component(*component_id).unwrap();
        }
        // And finally remove the entity from it's systems
        for system in self.systems.iter_mut() {
            if system.check_cbitfield(entity.cbitfield) {
                // Remove the entity, since it was contained in the system
                system.remove_entity(id);
            }
        }
        Ok(entity)
    }
    /* #endregion */
    /* #region Components */
    // Link some components to an entity
    pub(crate) fn link_components(&mut self, id: EntityID, link_group: ComponentLinkingGroup) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in link_group.linked_components {
            let idx = self.add_component(boxed, cbitfield)?;
            let entity = self.entity_mut(&id).unwrap();
            entity.components.push(idx);
        }
        // Change the entity's bitfield
        let entity = self.entity_mut(&id).unwrap();
        let cbitfield = entity.cbitfield.add(&link_group.cbitfield);
        entity.cbitfield = cbitfield; 
        // Check if the linked entity is valid to be added into the systems
        self.systems.iter_mut().for_each(|system| if system.check_cbitfield(cbitfield) {
            system.add_entity(id)
        });
        Ok(())
    }
    // Unlink some components from an entity
    pub(crate) fn unlink_components(&mut self, id: EntityID, unlink_group: ComponentUnlinkGroup) -> Result<(), ComponentError> {
        // Check if the entity even have these components
        let entity = self.entity(&id).unwrap();
        let valid = entity.cbitfield.contains(&unlink_group.removal_cbitfield);
        if !valid { return Err(ComponentError::new_without_id("The ComponentSplitGroup contains components that do not exist on the original entity!".to_string())) }
        // Remove the entity from some systems if needed
        let old = entity.cbitfield;
        let new = entity.cbitfield.remove(&unlink_group.removal_cbitfield).unwrap();
        self.systems.iter_mut().for_each(|system| {
            // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
            if system.check_cbitfield(old) && !system.check_cbitfield(new) {
                system.remove_entity(id);
            }
        });
        // Update the entity's components
        let entity = self.entity_mut(&id).unwrap();
        let components = entity.components.drain_filter(|component_id| unlink_group.removal_cbitfield.contains(&component_id.cbitfield)).collect::<Vec<_>>();
        entity.cbitfield = new;
        for component_id in components {
            self.remove_component(component_id)?;
        }
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError> {
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
        let idx = self
            .components_ids
            .remove(&id)
            .ok_or(ComponentError::new("Tried removing component, but it was not present in the ECS manager!".to_string(), id))?;
        self.components.remove(idx);
        Ok(())
    }
    /* #endregion */
    /* #region Systems */
    // Create a new system build
    pub fn create_system_builder<'a>(&'a mut self) -> SystemBuilder<'a, Context> {
        SystemBuilder::new(self)
    }
    // Add a system to our current systems
    pub(crate) fn add_system(&mut self, system: System) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn systems(&self) -> &[System] {
        self.systems.as_ref()
    }
    // Run the systems in sync, but their component updates is not
    // For now we will run them on the main thread, until I get my thread pool thingy working
    pub fn run_systems(&self, context: Context) where Context: Clone {
        for system in self.systems.iter() {
            system.run_system(context.clone(), self);
        }
    }
    /* #endregion */
    // Init update of the ECS manager
    pub fn init_update(&mut self) {
        self.entities.init_update();
    }
    // Finish update of the ECS manager
    pub fn finish_update(&mut self) {
        self.entities.finish_update();
    }
}
