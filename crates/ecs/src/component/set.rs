use ahash::AHashMap;
use bitfield::{AtomicSparseBitfield, Bitfield};
use slotmap::Key;
use std::{cell::UnsafeCell, sync::Arc};

use super::{registry, BoxedComponent, Component, ComponentGroupToRemove, ComponentKey, Components, DanglingComponentsToRemove, LinkedComponents};
use crate::{
    entity::{ComponentLinkingGroup, ComponentUnlinkGroup, EntityKey, EntitySet},
    system::SystemSet,
    utils::{ComponentError, ComponentLinkingError, ComponentUnlinkError},
};

// Component set
pub struct ComponentSet {
    // TODO: Use custom storage
    components: Components,
    pub(crate) to_remove: DanglingComponentsToRemove,
    pub(crate) mutated_components: Arc<AtomicSparseBitfield>,
}

impl Default for ComponentSet {
    fn default() -> Self {
        Self {
            components: Default::default(),
            to_remove: Default::default(),
            mutated_components: Default::default(),
        }
    }
}

// Errors
fn invalid_err() -> ComponentError {
    ComponentError::new("Component could not be fetched!".to_string())
}

impl ComponentSet {
    // Link some components to an entity
    pub fn link(&mut self, key: EntityKey, entities: &mut EntitySet, systems: &mut SystemSet, group: ComponentLinkingGroup) -> Result<(), ComponentLinkingError> {
        for (cbitfield, boxed) in group.linked {
            let (ckey, _ptr) = self.add(boxed);
            let entity = entities.get_mut(key).unwrap();
            entity.components.insert(cbitfield, ckey);
        }
        // Change the entity's bitfield
        let components = self.components.clone();
        let entity = entities.get_mut(key).unwrap();

        // Diff
        let old = entity.cbitfield;
        let new = entity.cbitfield.add(&group.cbitfield);
        entity.cbitfield = new;

        // Check if we already have some components linked to the entity
        if old.contains(&group.cbitfield) {
            return Err(ComponentLinkingError::new(
                "Cannot link components to entity because some have been already linked!".to_string(),
            ));
        }

        let entity = entities.get(key).unwrap();
        let linked = &entity.components;
        // Check if the linked entity is valid to be added into the subsystems
        let systems = systems.inner().borrow();
        for system in systems.iter() {
            for subsystem in system.subsystems.iter() {
                // If the entity wasn't inside the subsystem before we changed it's cbitfield, and it became valid afterwards, that means that we must add the entity to the subsystem
                if subsystem.check(new) && !subsystem.check(old) {
                    let linked = LinkedComponents {
                        key,
                        linked: linked.clone(),
                        mutated_components: self.mutated_components.clone(),
                        components: components.clone(),
                    };
                    subsystem.add(key, linked);
                }
            }
        }
        Ok(())
    }
    // Unlink some components from an entity
    pub fn unlink(&mut self, key: EntityKey, entities: &mut EntitySet, systems: &mut SystemSet, group: ComponentUnlinkGroup) -> Result<(), ComponentUnlinkError> {
        // Check if the entity even have these components
        let entity = entities.get(key).unwrap();
        let valid = entity.cbitfield.contains(&group.removal_cbitfield);
        if !valid {
            return Err(ComponentUnlinkError::new(
                "The ComponentUnlinkGroup contains components that do not exist on the original entity!".to_string(),
            ));
        }
        // Remove the entity from some systems if needed
        let old = entity.cbitfield;
        let new = entity.cbitfield.remove(&group.removal_cbitfield).unwrap();
        let systems = systems.inner().borrow();
        for system in systems.iter() {
            for subsystem in system.subsystems.iter() {
                // If the entity was inside the subsystem before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the subsystem
                if subsystem.check(old) && !subsystem.check(new) {
                    subsystem.remove(
                        key,
                        LinkedComponents {
                            key,
                            linked: entity.components.clone(),
                            mutated_components: self.mutated_components.clone(),
                            components: self.components.clone(),
                        },
                    );
                }
            }
        }
        // Update the entity's components
        let entity = entities.get_mut(key).unwrap();
        // Get the component keys that we must remove
        let components_elems = entity
            .components
            .iter()
            .filter_map(
                |(cbitfield, ckey)| {
                    if group.removal_cbitfield.contains(cbitfield) {
                        Some((*cbitfield, *ckey))
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        // We shall remove
        entity.cbitfield = new;
        let components = components_elems.iter().cloned().collect::<AHashMap<Bitfield<u32>, ComponentKey>>();
        for (cbitfield, _) in &components_elems {
            entity.components.remove(cbitfield).unwrap();
        }

        // And finally remove the component group from the required subsystems
        let mut lock = self.to_remove.borrow_mut();
        let counter = systems
            .iter()
            .flat_map(|systems| &systems.subsystems)
            .filter(|subsystem| {
                // If the entity was inside the subsystem before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the subsystem
                subsystem.check(old) && !subsystem.check(new)
            })
            .count();
        // Add the removal group
        lock.insert(ComponentGroupToRemove { components, counter, key });
        Ok(())
    }
    // Called at the start of the frame
    pub(crate) fn ready(&mut self, frame: u128) -> Result<(), ComponentError> {
        // Check if all the system have run the "Remove Entity" event, and if they did, we must internally remove the component group
        let removed_groups = {
            let mut lock = self.to_remove.borrow_mut();
            let indices = lock
                .iter()
                .filter_map(|(_key, group)| if group.counter == 0 { Some(_key) } else { None })
                .collect::<Vec<_>>();

            indices.into_iter().map(|x| lock.remove(x).unwrap()).collect::<Vec<ComponentGroupToRemove>>()
        };
        // Remove the dangling components
        for group in removed_groups {
            for (_, &key) in group.components.iter() {
                self.remove(key)?;
            }
        }
        // Also clear the bitfield indicating which components have been mutated
        if frame != 0 {
            self.mutated_components.clear();
        } else {
            // When the game starts, disable component registration
            registry::disable();
        }
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add(&mut self, boxed: BoxedComponent) -> (ComponentKey, *mut BoxedComponent) {
        // UnsafeCell moment
        let mut components = self.components.write();
        let cell = UnsafeCell::new(boxed);
        let ptr = cell.get();
        let key = components.insert(cell);
        let index = key.data().as_ffi() & 0xffff_ffff;
        self.mutated_components.set(index as usize, true);
        (key, ptr)
    }
    // Remove a specified component from the list
    fn remove(&mut self, key: ComponentKey) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        let mut components = self.components.write();
        components
            .remove(key)
            .ok_or_else(|| ComponentError::new("Tried removing component, but it was not present in the ECS manager!".to_string()))?;
        Ok(())
    }
    // Count the number of valid components in the ECS manager
    pub fn len(&self) -> usize {
        self.components.read().len()
    }

    // Get a single component
    pub fn get<T>(&self, key: ComponentKey) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Read directly
        let map = self.components.read();
        let cell = map.get(key).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast::<T>(component)?;
        Ok(component)
    }
    // Get a single component mutably
    pub fn get_mut<T>(&mut self, key: ComponentKey) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Read directly
        let map = self.components.read();
        let cell = map.get(key).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_mut::<T>(component)?;
        // We only care about the index
        let index = key.data().as_ffi() & 0xffff_ffff;
        self.mutated_components.set(index as usize, true);
        Ok(component)
    }
}
