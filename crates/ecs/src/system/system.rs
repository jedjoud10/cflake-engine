use std::{cell::RefCell, rc::Rc};

use ahash::AHashMap;
use bitfield::Bitfield;

use super::SystemExecutionData;
use crate::{
    component::{ComponentQuery, LinkedComponents},
    entity::EntityID,
    event::EventKey,
    ECSManager,
};

// A system that updates specific components in parallel
#[derive(Default)]
pub struct System {
    pub(crate) cbitfield: Bitfield<u32>,
    // Events
    pub(crate) evn_run: Option<usize>,
    pub(crate) evn_added_entity: Option<usize>,
    pub(crate) evn_removed_entity: Option<usize>,

    linked_components: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
    // Added, Removed
    added: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
    removed: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
}

// System code
impl System {
    // Check if an entity validates our cbitfield
    pub(crate) fn check_cbitfield(&self, cbitfield: Bitfield<u32>) -> bool {
        cbitfield.contains(&self.cbitfield)
    }
    // Add an entity
    pub(crate) fn add_entity(&self, id: EntityID, linked_components: LinkedComponents) {
        let cloned = LinkedComponents {
            components: linked_components.components.clone(),
            mutated_components: linked_components.mutated_components.clone(),
            linked: linked_components.linked.clone(),
            id: linked_components.id.clone(),
        };
        let mut lock = self.linked_components.borrow_mut();
        lock.insert(id, linked_components);
        let mut lock = self.added.borrow_mut();
        lock.insert(id, cloned);
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&self, id: EntityID, linked_components: LinkedComponents) {
        let mut lock = self.linked_components.borrow_mut();
        if lock.contains_key(&id) {
            lock.remove(&id);
            let mut removed_lock = self.removed.borrow_mut();
            removed_lock.insert(id, linked_components);
        }
    }
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system<World>(&self, ecs_manager: &ECSManager<World>) -> SystemExecutionData<World> {
        // Create the component queries
        let all_components = self.evn_run.map(|_| self.linked_components.clone());

        // Get the added components
        let added_components = self.evn_added_entity.map(|_| self.added.clone());

        // Do a bit of decrementing
        let removed_components = {
            let removed = self.removed.borrow_mut();
            let mut lock = ecs_manager.component_groups_to_remove.lock();
            for (_, components) in lock.iter_mut() {
                if removed.contains_key(&components.entity_id) {
                    // Decrement
                    components.counter -= 1;
                }
            }
            self.evn_removed_entity.map(|_| self.removed.clone())
        };
        SystemExecutionData {
            // Events
            evn_run: ecs_manager.event_handler.get_run_event(self.evn_run).cloned(),
            evn_added_entity: ecs_manager.event_handler.get_added_entity_event(self.evn_added_entity).cloned(),
            evn_removed_entity: ecs_manager.event_handler.get_removed_entity_event(self.evn_removed_entity).cloned(),
            // Queries
            evn_run_ekey: EventKey::Query(ComponentQuery {
                linked_components: all_components,
            }),
            evn_added_entity_ekey: EventKey::Query(ComponentQuery {
                linked_components: added_components,
            }),
            evn_removed_entity_ekey: EventKey::Query(ComponentQuery {
                linked_components: removed_components,
            }),
        }
    }
    // Clear the system for the next execution
    pub fn clear<World>(&self) {
        // Clear the stored entity differences
        let mut added = self.added.borrow_mut();
        added.clear();
        let mut removed = self.removed.borrow_mut();
        removed.clear();
    }
}
