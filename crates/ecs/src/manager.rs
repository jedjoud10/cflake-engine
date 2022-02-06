use std::{
    any::TypeId,
    cell::UnsafeCell,
    sync::{Arc, Mutex, RwLock},
};

use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::{shareable::ShareableOrderedVec, simple::OrderedVec};
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::{
    component::{ComponentID, EnclosedComponent, LinkedComponents},
    entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity, EntityID},
    system::{System, SystemBuilder},
    utils::{ComponentError, EntityError}, event::EventHandler,
};

// The Entity Component System manager that will handle everything ECS related
pub struct ECSManager<World> {
    // A vector full of entitie
    pub(crate) entities: ShareableOrderedVec<Entity>,
    pub(crate) entities_to_remove: Mutex<OrderedVec<(Entity, u64, usize)>>,
    // Each system, stored in the order they were created
    pub(crate) systems: Vec<System>,
    // The components that are valid in the world
    pub(crate) components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>,
    // The internal ECS thread pool
    pub(crate) rayon_pool: Arc<ThreadPool>,
    // Our internal event handler
    pub(crate) event_handler: EventHandler<World>,
}

// Global code for the Entities, Components, and Systems
impl<World> ECSManager<World> {
    // Create a new ECS manager
    pub fn new() -> Self {
        // Start the rayon thread pool
        let pool = ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        Self {
            entities: Default::default(),
            entities_to_remove: Default::default(),
            systems: Default::default(),
            components: Default::default(),
            rayon_pool: Arc::new(pool),
            event_handler: Default::default(),
        }
    }
    /* #region Entities */
    // Get an entity
    pub fn get_entity(&self, id: &EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.0).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Get an entity mutably
    pub fn get_entity_mut(&mut self, id: &EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.0).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add_entity(&mut self, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup) -> Result<(), EntityError> {
        // Check if the EntityID was not occupied already
        if self.entities.get(id.0).is_some() {
            return Err(EntityError::new("Tried adding entity, but the EntityID was already occupied!".to_string(), id));
        }
        entity.id = Some(id);
        // Add the entity
        let _idx = self.entities.insert(id.0, entity);
        // After doing that, we can safely add the components
        self.link_components(id, group).unwrap();
        Ok(())
    }
    // Remove an entity, but keep it's components alive until all systems have been notified
    pub fn remove_entity(&mut self, id: EntityID) -> Result<(), EntityError> {
        // Invalidate the entity
        let entity = self.entities.remove(id.0).ok_or(EntityError::new("Could not find entity!".to_string(), id))?;
        let cbitfield = entity.cbitfield;
        // And finally remove the entity from it's systems
        let mut lock = self.entities_to_remove.lock().unwrap();
        let removed_id = lock.get_next_id();
        lock.push_shove((entity, removed_id, 0));
        // Get the pointer to the new entity components

        for system in self.systems.iter_mut() {
            if system.check_cbitfield(cbitfield) {
                // Remove the entity, since it was contained in the system
                let (entity, _, counter) = lock.get_mut(removed_id).unwrap();
                system.remove_entity(id, LinkedComponents::new_dead(removed_id, &entity.components, self.components.clone()));
                *counter += 1;
            }
        }
        Ok(())
    }
    // Remove the dangling components that have been linked to an entity that we already removed
    fn remove_dangling_components(&mut self, entity: &Entity) -> Result<(), ComponentError> {
        // Also remove it's linked components
        for (cbitfield, idx) in entity.components.iter() {
            self.remove_component(ComponentID::new(*cbitfield, *idx))?;
        }
        Ok(())
    }
    // Count the number of valid entities in the ECS manager
    pub fn count_entities(&self) -> usize {
        self.entities.count()
    }
    /* #endregion */
    /* #region Components */
    // Link some components to an entity
    pub fn link_components(&mut self, id: EntityID, link_group: ComponentLinkingGroup) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in link_group.linked_components {
            let (component_id, _ptr) = self.add_component(boxed, cbitfield)?;
            let entity = self.get_entity_mut(&id).unwrap();
            entity.components.insert(cbitfield, component_id.idx);
        }
        // Change the entity's bitfield
        let components = self.components.clone();
        let entity = self.get_entity_mut(&id).unwrap();

        // Diff
        let old = entity.cbitfield;
        let new = entity.cbitfield.add(&link_group.cbitfield);
        entity.cbitfield = new;

        let entity = self.get_entity(&id).unwrap();
        let linked = &entity.components;
        // Check if the linked entity is valid to be added into the systems
        for system in self.systems.iter() {
            // If the entity wasn't inside the system before we changed it's cbitfield, and it became valid afterwards, that means that we must add the entity to the system
            if system.check_cbitfield(new) && !system.check_cbitfield(old) {
                let lc1 = LinkedComponents::new_direct(id, linked, components.clone());
                let lc2 = LinkedComponents::new_direct(id, linked, components.clone());
                system.add_entity(id, lc1, lc2);
            }
        }
        Ok(())
    }
    // Unlink some components from an entity
    pub fn unlink_components(&mut self, id: EntityID, unlink_group: ComponentUnlinkGroup) -> Result<(), ComponentError> {
        // Check if the entity even have these components
        let entity = self.get_entity(&id).unwrap();
        let valid = entity.cbitfield.contains(&unlink_group.removal_cbitfield);
        if !valid {
            return Err(ComponentError::new_without_id(
                "The ComponentSplitGroup contains components that do not exist on the original entity!".to_string(),
            ));
        }
        // Remove the entity from some systems if needed
        let old = entity.cbitfield;
        let new = entity.cbitfield.remove(&unlink_group.removal_cbitfield).unwrap();
        self.systems.iter().for_each(|system| {
            // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
            if system.check_cbitfield(old) && !system.check_cbitfield(new) {
                system.remove_entity(id, LinkedComponents::new(entity, self.components.clone()));
            }
        });
        // Update the entity's components
        let entity = self.get_entity_mut(&id).unwrap();
        let components = entity
            .components
            .drain_filter(|cbitfield, _idx| unlink_group.removal_cbitfield.contains(cbitfield))
            .collect::<Vec<_>>();
        entity.cbitfield = new;
        for (cbitfield, idx) in components {
            self.remove_component(ComponentID::new(cbitfield, idx))?;
        }
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<(ComponentID, *mut EnclosedComponent), ComponentError> {
        // UnsafeCell moment
        let mut components = self.components.write().unwrap();
        let cell = UnsafeCell::new(boxed);
        let ptr = cell.get();
        let idx = components.push_shove(cell);
        // Create a new Component ID
        let id = ComponentID::new(cbitfield, idx);
        Ok((id, ptr))
    }
    // Remove a specified component from the list
    fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        let mut components = self.components.write().unwrap();
        components
            .remove(id.idx)
            .ok_or(ComponentError::new("Tried removing component, but it was not present in the ECS manager!".to_string(), id))?;
        Ok(())
    }
    // Count the number of valid components in the ECS manager
    pub fn count_components(&self) -> usize {
        self.components.read().unwrap().count()
    }
    /* #endregion */
    /* #region Systems */
    // Create a new system build
    pub fn create_system_builder<'a>(&'a mut self) -> SystemBuilder<'a, World> {
        SystemBuilder::new(self)
    }
    // Add a system to our current systems
    pub(crate) fn add_system(&mut self, system: System) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn get_systems(&self) -> &[System] {
        self.systems.as_ref()
    }
    // Get the number of systems that we have
    pub fn count_systems(&self) -> usize {
        self.systems.len()
    }
    // Run the systems in sync, but their component updates are not
    // Used only for testing
    #[allow(dead_code)]
    pub(crate) fn run_systems(&self, world: &mut World) {
        for system in self.systems.iter() {
            let execution_data = system.run_system(self);
            execution_data.run(world);
            system.clear::<World>();
        }
    }
    /* #endregion */
    // Finish update of the ECS manager
    pub fn finish_update(&mut self) {
        // Check if all the system have run the "Remove Entity" event, and if they did, we must internally remove the entity
        let removed_entities = {
            let mut lock = self.entities_to_remove.lock().unwrap();
            lock.my_drain(|_, (_, _, count)| *count == 0).collect::<Vec<_>>()
        };
        // Remove the dangling components
        for (_, (entity, _, _)) in removed_entities {
            self.remove_dangling_components(&entity).unwrap();
        }
    }
}
