use ahash::AHashMap;
use bitfield::{AtomicSparseBitfield, Bitfield};
use ordered_vec::simple::OrderedVec;
use parking_lot::Mutex;
use std::{cell::UnsafeCell, sync::Arc};

use crate::{
    component::{ComponentGroupToRemove, ComponentID, ComponentsCollection, EnclosedComponent, LinkedComponents},
    entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity, EntityID},
    system::{System, SystemBuilder},
    utils::{ComponentError, ComponentLinkingError, ComponentUnlinkError, EntityError},
};

// The Entity Component System manager that will handle everything ECS related
pub struct ECSManager<World> {
    // A vector full of entitie
    pub(crate) entities: OrderedVec<Entity>,
    pub(crate) component_groups_to_remove: Mutex<OrderedVec<ComponentGroupToRemove>>,
    // Each system, stored in the order they were created
    pub(crate) systems: Vec<System<World>>,
    // The components that are valid in the world
    pub(crate) components: ComponentsCollection,
    pub(crate) mutated_components: Arc<AtomicSparseBitfield>,
}

impl<World> Default for ECSManager<World> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            component_groups_to_remove: Default::default(),
            systems: Default::default(),
            components: Default::default(),
            mutated_components: Default::default(),
        }
    }
}

// Global code for the Entities, Components, and Systems
impl<World> ECSManager<World> {
    /* #region Entities */
    // Get an entity
    pub fn get_entity(&self, id: &EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.0).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Get an entity mutably
    pub fn get_entity_mut(&mut self, id: &EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.0).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add_entity(&mut self, mut entity: Entity, group: ComponentLinkingGroup) -> Result<EntityID, EntityError> {
        // Get the ID
        let id = EntityID(self.entities.get_next_id());
        entity.id = Some(id);
        // Add the entity
        let _idx = self.entities.push_shove(entity);
        // After doing that, we can safely add the components
        self.link_components(id, group).unwrap();
        Ok(id)
    }
    // Remove an entity, but keep it's components alive until all systems have been notified
    pub fn remove_entity(&mut self, id: EntityID) -> Result<(), EntityError> {
        let entity = self.entities.get(id.0).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), id))?;
        let group = ComponentUnlinkGroup::unlink_all_from_entity(entity);
        // Unlink all of the entity's components
        self.unlink_components(id, group).unwrap();
        // Invalidate the entity
        let _entity = self.entities.remove(id.0).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), id))?;
        Ok(())
    }
    // Remove the dangling components that have been linked to a group
    fn remove_dangling_components(&mut self, group: &ComponentGroupToRemove) -> Result<(), ComponentError> {
        // Also remove it's linked components
        for (cbitfield, idx) in group.components.iter() {
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
    pub fn link_components(&mut self, id: EntityID, link_group: ComponentLinkingGroup) -> Result<(), ComponentLinkingError> {
        for (cbitfield, boxed) in link_group.linked_components {
            let (component_id, _ptr) = self.add_component(boxed, cbitfield);
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

        // Check if we already have some components linked to the entity
        if old.contains(&link_group.cbitfield) {
            return Err(ComponentLinkingError::new(
                "Cannot link components to entity because some have been already linked!".to_string(),
            ));
        }

        let entity = self.get_entity(&id).unwrap();
        let linked = &entity.components;
        // Check if the linked entity is valid to be added into the systems
        for system in self.systems.iter() {
            // If the entity wasn't inside the system before we changed it's cbitfield, and it became valid afterwards, that means that we must add the entity to the system
            if system.check_cbitfield(new) && !system.check_cbitfield(old) {
                let linked = LinkedComponents {
                    id,
                    linked: linked.clone(),
                    mutated_components: self.mutated_components.clone(),
                    components: components.clone(),
                };
                system.add_entity(id, linked);
            }
        }
        Ok(())
    }
    // Unlink some components from an entity
    pub fn unlink_components(&mut self, id: EntityID, unlink_group: ComponentUnlinkGroup) -> Result<(), ComponentUnlinkError> {
        // Check if the entity even have these components
        let entity = self.get_entity(&id).unwrap();
        let valid = entity.cbitfield.contains(&unlink_group.removal_cbitfield);
        if !valid {
            return Err(ComponentUnlinkError::new(
                "The ComponentUnlinkGroup contains components that do not exist on the original entity!".to_string(),
            ));
        }
        // Remove the entity from some systems if needed
        let old = entity.cbitfield;
        let new = entity.cbitfield.remove(&unlink_group.removal_cbitfield).unwrap();
        self.systems.iter().for_each(|system| {
            // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
            if system.check_cbitfield(old) && !system.check_cbitfield(new) {
                system.remove_entity(
                    id,
                    LinkedComponents {
                        id,
                        linked: entity.components.clone(),
                        mutated_components: self.mutated_components.clone(),
                        components: self.components.clone(),
                    },
                );
            }
        });
        // Update the entity's components
        let entity = self.get_entity_mut(&id).unwrap();
        // Dear god
        let components_elems = entity
            .components
            .iter()
            .filter_map(|(cbitfield, idx)| {
                if unlink_group.removal_cbitfield.contains(cbitfield) {
                    Some((*cbitfield, *idx))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // We shall remove
        entity.cbitfield = new;
        let components = components_elems.iter().cloned().collect::<AHashMap<Bitfield<u32>, u64>>();
        for (cbitfield, _) in &components_elems {
            entity.components.remove(cbitfield).unwrap();
        }
        drop(entity);

        // And finally remove the component group from it's systems
        let mut lock = self.component_groups_to_remove.lock();
        let counter = self
            .systems
            .iter()
            .filter(|system| {
                // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
                system.check_cbitfield(old) && !system.check_cbitfield(new)
            })
            .count();
        lock.push_shove(ComponentGroupToRemove {
            components: components.clone(),
            counter,
            entity_id: id,
        });
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> (ComponentID, *mut EnclosedComponent) {
        // UnsafeCell moment
        let mut components = self.components.write();
        let cell = UnsafeCell::new(boxed);
        let ptr = cell.get();
        let idx = components.push_shove(cell);
        // Create a new Component ID
        let id = ComponentID::new(cbitfield, idx);
        (id, ptr)
    }
    // Remove a specified component from the list
    fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        let mut components = self.components.write();
        components
            .remove(id.idx)
            .ok_or_else(|| ComponentError::new("Tried removing component, but it was not present in the ECS manager!".to_string()))?;
        Ok(())
    }
    // Count the number of valid components in the ECS manager
    pub fn count_components(&self) -> usize {
        self.components.read().count()
    }
    /* #endregion */
    /* #region Systems */
    // Create a new system build
    pub fn build_system(&mut self) -> SystemBuilder<World> {
        SystemBuilder::new(self)
    }
    // Add a system to our current systems
    pub(crate) fn add_system(&mut self, system: System<World>) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn get_systems(&self) -> &[System<World>] {
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
            system.clear();
        }
    }
    /* #endregion */
    // Finish update of the ECS manager
    pub fn finish_update(&mut self) {
        // Check if all the system have run the "Remove Entity" event, and if they did, we must internally remove the component group
        let removed_group = {
            let mut lock = self.component_groups_to_remove.lock();
            lock.my_drain(|_, group| group.counter == 0).collect::<Vec<_>>()
        };
        // Remove the dangling components
        for (_, group) in removed_group {
            self.remove_dangling_components(&group).unwrap();
        }
        // Also clear the bitfield indicating which components have been mutated
        self.mutated_components.clear();
    }
}
