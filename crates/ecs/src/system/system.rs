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

pub(crate) type Event<World> = Option<fn(&mut World, EventKey)>;

// A system that updates specific components in parallel
pub struct System<World> {
    pub(crate) cbitfield: Bitfield<u32>,
    // Events
    pub(crate) evn_run: Event<World>,
    pub(crate) evn_run_fixed: Event<World>,
    pub(crate) evn_added_entity: Event<World>,
    pub(crate) evn_removed_entity: Event<World>,

    linked_components: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
    // Added, Removed
    added: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
    removed: Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>,
}

impl<World> Default for System<World> {
    fn default() -> Self {
        Self {
            cbitfield: Default::default(),
            evn_run: Default::default(),
            evn_run_fixed: Default::default(),
            evn_added_entity: Default::default(),
            evn_removed_entity: Default::default(),
            linked_components: Default::default(),
            added: Default::default(),
            removed: Default::default(),
        }
    }
}

// System code
impl<World> System<World> {
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
    pub fn run_system(&self, ecs_manager: &ECSManager<World>) -> SystemExecutionData<World> {
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
            run: (
                self.evn_run,
                EventKey::Query(ComponentQuery {
                    linked_components: all_components,
                }),
            ),
            added_entity: (
                self.evn_added_entity,
                EventKey::Query(ComponentQuery {
                    linked_components: added_components,
                }),
            ),
            removed_entity: (
                self.evn_removed_entity,
                EventKey::Query(ComponentQuery {
                    linked_components: removed_components,
                }),
            ),
        }
    }
    // Clear the system for the next execution
    pub fn clear(&self) {
        // Clear the stored entity differences
        let mut added = self.added.borrow_mut();
        added.clear();
        let mut removed = self.removed.borrow_mut();
        removed.clear();
    }
}
