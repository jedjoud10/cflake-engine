use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex},
};

use ahash::{AHashMap, AHashSet};
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;

use super::SystemExecutionData;
use crate::{
    component::{ComponentQuery, ComponentQueryIterType, EnclosedComponent, LinkedComponents},
    entity::{Entity, EntityID},
    ECSManager,
};

// A system that updates specific components in parallel
pub struct System {
    pub(crate) cbitfield: Bitfield<u32>,
    // Events
    pub(crate) evn_run: Option<usize>,
    pub(crate) evn_added_entity: Option<usize>,
    pub(crate) evn_removed_entity: Option<usize>,

    linked_components: Arc<Mutex<AHashMap<EntityID, LinkedComponents>>>,
    // Added, Removed
    added: Mutex<Vec<LinkedComponents>>,
    removed: Mutex<Vec<LinkedComponents>>,
}

impl Default for System {
    fn default() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            evn_run: None,
            evn_added_entity: None,
            evn_removed_entity: None,
            linked_components: Default::default(),
            added: Default::default(),
            removed: Default::default(),
        }
    }
}

// System code
impl System {
    // Check if an entity validates our cbitfield
    pub(crate) fn check_cbitfield(&self, cbitfield: Bitfield<u32>) -> bool {
        cbitfield.contains(&self.cbitfield)
    }
    // Add an entity
    pub(crate) fn add_entity(&self, id: EntityID, linked_components: LinkedComponents, linked_components2: LinkedComponents) {
        let mut lock = self.linked_components.lock().unwrap();
        let id = lock.insert(id, linked_components);
        let mut lock = self.added.lock().unwrap();
        lock.push(linked_components2);
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&self, id: EntityID, linked_components: LinkedComponents) {
        let mut lock = self.linked_components.lock().unwrap();
        if lock.contains_key(&id) {
            lock.remove(&id);
            let mut removed_lock = self.removed.lock().unwrap();
            removed_lock.push(linked_components);
        }        
    }
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system<Context>(&self, ecs_manager: &ECSManager<Context>) -> SystemExecutionData<Context> {
        // Create the component queries
        let all_components = self.evn_run.map(|_| ComponentQueryIterType::ArcHashMap(self.linked_components.clone()));


        // Get the added components
        let added_components = {
            // We must ALWAYS take it
            let mut added = self.added.lock().unwrap();
            let vec = std::mem::take(&mut *added);
            self.evn_added_entity.map(|_| ComponentQueryIterType::Vec(vec))
        };

        // Do a bit of decrementing
        let removed_components = {
            let mut removed = self.removed.lock().unwrap();
            let mut lock = ecs_manager.entities_to_remove.lock().unwrap();
            for component in removed.iter() {
                // Decrement the counter
                let (_entity, _removed_id, counter) = lock.get_mut(component.id).unwrap();
                *counter -= 1;
            }
            // Clear the "removed" vector and return it's elements
            let vec = std::mem::take(&mut *removed);
            self.evn_removed_entity.map(|_| ComponentQueryIterType::Vec(vec))
        };
        SystemExecutionData {
            // Events
            evn_run: ecs_manager.event_handler.get_run_event(self.evn_run).cloned(),
            evn_added_entity: ecs_manager.event_handler.get_added_entity_event(self.evn_added_entity).cloned(),
            evn_removed_entity: ecs_manager.event_handler.get_removed_entity_event(self.evn_removed_entity).cloned(),
            // Queries
            evn_run_query: ComponentQuery {
                linked_components: all_components,
                thread_pool: ecs_manager.thread_pool.clone(),
            },
            evn_added_entity_query: ComponentQuery {
                linked_components: added_components,
                thread_pool: ecs_manager.thread_pool.clone(),
            },
            evn_removed_entity_query: ComponentQuery {
                linked_components: removed_components,
                thread_pool: ecs_manager.thread_pool.clone(),
            },
        }
    }
}
