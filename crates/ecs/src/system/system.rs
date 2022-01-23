use std::{cell::UnsafeCell, sync::Mutex};

use ahash::{AHashMap, AHashSet};
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;

use super::{SystemExecutionData};
use crate::{
    component::{ComponentQuery, EnclosedComponent, LinkedComponents, StoredGlobalComponents},
    entity::{Entity, EntityID},
    ECSManager,
};

// A system that updates specific components in parallel
pub struct System {
    pub(crate) cbitfield: Bitfield<u32>,
    // A bitfield indicating what global components we have access to
    pub(crate) global_component_access_cbitfield: Bitfield<u32>,
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
            global_component_access_cbitfield: Bitfield::<u32>::default(),
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
    // Update our global component cbitfield so we can access (read/write) the component
    pub(crate) fn add_access_state(&mut self, cbitfield: Bitfield<u32>) {
        self.global_component_access_cbitfield = self.global_component_access_cbitfield.add(&cbitfield);
    }
    // Create a SystemExecutionData that we can actually run at a later time
    // TODO: Optimize this shit
    pub fn run_system<Context>(&self, ecs_manager: &ECSManager<Context>) -> SystemExecutionData<Context> {
        // These components are filtered for us
        let components = &ecs_manager.components.lock().unwrap();
        // Create the component queries
        let all_components = Self::get_linked_components(&self.evn_run, components, self.entities.iter().cloned(), ecs_manager);
        let mut lock = self.changed_entities.lock().unwrap();
        let removed_entities = lock.1.drain();
        // We must decrement the counter for each removed entity
        let mut entities_to_remove_ecs_manager = ecs_manager.entities_to_remove.lock().unwrap();
        let removed_components = Self::get_linked_components_removed(&mut *entities_to_remove_ecs_manager, &self.evn_removed_entity, components, removed_entities, ecs_manager);
        let added_entities = lock.0.drain();
        let added_components = Self::get_linked_components(&self.evn_added_entity, components, added_entities, ecs_manager);
        // Get the global components for this system. We can cache this later
        let stored_global_components = if self.global_component_access_cbitfield.empty() { None }
        else { Some(StoredGlobalComponents::new(self.global_component_access_cbitfield, &ecs_manager)) };
        SystemExecutionData {
            // Events
            evn_run: ecs_manager.event_handler.get_run_event(self.evn_run).cloned(),
            evn_added_entity: ecs_manager.event_handler.get_added_entity_event(self.evn_added_entity).cloned(),
            evn_removed_entity: ecs_manager.event_handler.get_removed_entity_event(self.evn_removed_entity).cloned(),
            // Queries
            evn_run_query: ComponentQuery {
                linked_components: all_components,
                thread_pool: ecs_manager.thread_pool.clone(),
                stored_global_components: stored_global_components.clone()
            },
            evn_added_entity_query: ComponentQuery {
                linked_components: added_components,
                thread_pool: ecs_manager.thread_pool.clone(),
                stored_global_components: stored_global_components.clone()
            },
            evn_removed_entity_query: ComponentQuery {
                linked_components: removed_components,
                thread_pool: ecs_manager.thread_pool.clone(),
                stored_global_components: stored_global_components
            },
        }
    }

    // Get linked components for a vector full of entity IDs
    fn get_linked_components<Context, T: Iterator<Item = EntityID>>(
        evn: &Option<usize>,
        components: &OrderedVec<UnsafeCell<EnclosedComponent>>,
        entities: T,
        ecs_manager: &ECSManager<Context>,
    ) -> Option<Vec<LinkedComponents>> {
        if evn.is_some() {
            let components = entities
                .map(|id| {
                    let entity = ecs_manager.entity(&id).unwrap();

                    LinkedComponents::new(entity, components)
                })
                .collect::<Vec<_>>();
            Some(components)
        } else {
            None
        }
    }

    // Get linked components for removed entities that we must call their removed event
    fn get_linked_components_removed<Context, T: Iterator<Item = EntityID>>(
        lock: &mut AHashMap<EntityID, (Entity, usize)>,
        evn: &Option<usize>,
        components: &OrderedVec<UnsafeCell<EnclosedComponent>>,
        entities: T,
        _ecs_manager: &ECSManager<Context>,
    ) -> Option<Vec<LinkedComponents>> {
        if evn.is_some() {
            let components = entities
                .map(|id| {
                    // Decrement the counter
                    let (_entity, counter) = lock.get_mut(&id).unwrap();
                    *counter -= 1;
                    let (entity, _count) = lock.get(&id).unwrap();

                    LinkedComponents::new(entity, components)
                })
                .collect::<Vec<_>>();
            Some(components)
        } else {
            None
        }
    }
}
