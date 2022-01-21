use std::{cell::UnsafeCell, sync::Mutex};

use ahash::AHashSet;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;

use super::{EventHandler, SystemExecutionData};
use crate::{
    component::{registry, Component, ComponentQuery, LinkedComponents, EnclosedComponent},
    entity::EntityID,
    ECSManager,
};

// A system that updates specific components in parallel
pub struct System {
    pub(crate) cbitfield: Bitfield<u32>,
    // Events
    pub(crate) evn_run: Option<usize>,
    pub(crate) evn_added_entity: Option<usize>,
    pub(crate) evn_removed_entity: Option<usize>,

    entities: AHashSet<EntityID>,
    // Added, Removed
    changed_entities: Mutex<(AHashSet<EntityID>, AHashSet<EntityID>)>,
}

impl Default for System {
    fn default() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            evn_run: None,
            evn_added_entity: None,
            evn_removed_entity: None,
            entities: AHashSet::new(),
            changed_entities: Mutex::new((AHashSet::new(), AHashSet::new())),
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
    pub(crate) fn add_entity(&mut self, id: EntityID) {
        self.entities.insert(id);
        let mut lock = self.changed_entities.lock().unwrap();
        lock.0.insert(id);
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&mut self, id: EntityID) {
        if self.entities.contains(&id) {
            self.entities.remove(&id);
            let mut lock = self.changed_entities.lock().unwrap();
            lock.1.insert(id);
        }
    }
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system<Context>(&self, ecs_manager: &ECSManager<Context>) -> SystemExecutionData<Context> {
        // These components are filtered for us
        let components = &ecs_manager.components.lock().unwrap();
        // Create the component queries
        let all_components = Self::get_linked_components(&self.evn_run, components, self.entities.iter().cloned(), ecs_manager);
        let mut lock = self.changed_entities.lock().unwrap();        
        let removed_entities = lock.1.drain();
        // We must decrement the counter for each removed entity
        let mut entities_to_remove_ecs_manager = ecs_manager.entities_to_remove.lock().unwrap();
        let removed_entities = removed_entities.map(|x| {
            // Decrement the counter
            let counter = entities_to_remove_ecs_manager.get_mut(&x).unwrap();
            *counter -= 1;
            x
        });
        let removed_components = Self::get_linked_components(&self.evn_removed_entity, components, removed_entities, ecs_manager);
        let added_entities = lock.0.drain();
        let added_components = Self::get_linked_components(&self.evn_added_entity, components, added_entities, ecs_manager);
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
            } 
        }
    }

    // Get linked components for a vector full of entity IDs
    fn get_linked_components<Context, T: Iterator<Item = EntityID>>(evn: &Option<usize>, components: &OrderedVec<UnsafeCell<EnclosedComponent>>, entities: T, ecs_manager: &ECSManager<Context>) -> Option<Vec<LinkedComponents>> {
        if evn.is_some() { 
            let components = entities
            .map(|id| {
                let entity = ecs_manager.entity(&id).unwrap();
                let linked_components = LinkedComponents::new(entity, components);
                linked_components
            })
            .collect::<Vec<_>>();
            Some(components)
        } else { None }
    }
}
