use std::{marker::PhantomData, sync::Arc, cell::{UnsafeCell, RefCell}, rc::Rc};
use ahash::AHashMap;
use bitfield::{Bitfield, AtomicSparseBitfield};
use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::{entity::{EntityKey, ComponentLinkingGroup, EntitySet, ComponentUnlinkGroup}, utils::{ComponentLinkingError, ComponentUnlinkError, ComponentError}, system::SystemSet};
use super::{Components, LinkedComponents, ComponentKey, ComponentGroupToRemove, EnclosedComponent, ComponentGroupKey, DanglingComponentsToRemove};

// Component set
pub struct ComponentSet<World> {
    pub(crate) components: Components,
    pub(crate) _phantom: PhantomData<World>,
    pub(crate) to_remove: DanglingComponentsToRemove,
    pub(crate) mutated_components: Arc<AtomicSparseBitfield>,
}

impl<World> Default for ComponentSet<World> {
    fn default() -> Self {
        Self { 
            components: Default::default(),
            _phantom: Default::default(),
            to_remove: Default::default(),
            mutated_components: Default::default(),
        }
    }
}

impl<World> ComponentSet<World> {
    // Link some components to an entity
    pub fn link(&mut self, key: EntityKey, entities: &mut EntitySet<World>, systems: &mut SystemSet<World>, group: ComponentLinkingGroup) -> Result<(), ComponentLinkingError> {
        for (cbitfield, boxed) in group.linked_components {
            let (ckey, _ptr) = self.add(boxed, cbitfield);
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
        // Check if the linked entity is valid to be added into the systems
        let systems = systems.inner().borrow();
        for system in systems.iter() {
            // If the entity wasn't inside the system before we changed it's cbitfield, and it became valid afterwards, that means that we must add the entity to the system
            if system.check_cbitfield(new) && !system.check_cbitfield(old) {
                let linked = LinkedComponents {
                    key,
                    linked: linked.clone(),
                    mutated_components: self.mutated_components.clone(),
                    components: components.clone(),
                };
                system.add_entity(key, linked);
            }
        }
        Ok(())
    }
    // Unlink some components from an entity
    pub fn unlink(&mut self, key: EntityKey, entities: &mut EntitySet<World>, systems: &mut SystemSet<World>, group: ComponentUnlinkGroup) -> Result<(), ComponentUnlinkError> {
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
        systems.iter().for_each(|system| {
            // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
            if system.check_cbitfield(old) && !system.check_cbitfield(new) {
                system.remove_entity(
                    key,
                    LinkedComponents {
                        key,
                        linked: entity.components.clone(),
                        mutated_components: self.mutated_components.clone(),
                        components: self.components.clone(),
                    },
                );
            }
        });
        // Update the entity's components
        let entity = entities.get_mut(key).unwrap();
        // Dear god
        let components_elems = entity
            .components
            .iter()
            .filter_map(|(cbitfield, ckey)| {
                if group.removal_cbitfield.contains(cbitfield) {
                    Some((*cbitfield, *ckey))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // We shall remove
        entity.cbitfield = new;
        let components = components_elems.iter().cloned().collect::<AHashMap<Bitfield<u32>, ComponentKey>>();
        for (cbitfield, _) in &components_elems {
            entity.components.remove(cbitfield).unwrap();
        }
        drop(entity);

        // And finally remove the component group from it's systems
        let mut lock = self.to_remove.borrow_mut();
        let counter = systems
            .iter()
            .filter(|system| {
                // If the entity was inside the system before we changed it's cbitfield, and it became invalid afterwards, that means that we must remove the entity from the system
                system.check_cbitfield(old) && !system.check_cbitfield(new)
            })
            .count();
        lock.insert(ComponentGroupToRemove {
            components: components.clone(),
            counter,
            key,
        });
        Ok(())
    }    
    // Called at the end of the frame to clear the per frame values
    pub(crate) fn clear(&mut self) -> Result<(), ComponentError> {
        // Check if all the system have run the "Remove Entity" event, and if they did, we must internally remove the component group
        let removed_groups = {
            let mut lock = self.to_remove.borrow_mut();
            let indices = lock.iter().filter_map(|(_key, group)| if group.counter == 0 { Some(_key) } else { None }).collect::<Vec<_>>();
            let removed_groups = indices.into_iter().map(|x| lock.remove(x).unwrap()).collect::<Vec<ComponentGroupToRemove>>();
            removed_groups
        };
        // Remove the dangling components
        for group in removed_groups {
            for (_, &key) in group.components.iter() {
                self.remove(key)?;
            }
        }
        // Also clear the bitfield indicating which components have been mutated
        self.mutated_components.clear();        
        Ok(())
    }    
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add(&mut self, boxed: EnclosedComponent, _: Bitfield<u32>) -> (ComponentKey, *mut EnclosedComponent) {
        // UnsafeCell moment
        let mut components = self.components.write();
        let cell = UnsafeCell::new(boxed);
        let ptr = cell.get();
        let key = components.insert(cell);
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
}