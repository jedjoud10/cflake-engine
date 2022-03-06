use std::{cell::RefCell, rc::Rc};

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{ComponentQuery, LinkedComponents, DanglingComponentsToRemove},
    event::EventKey,
    ECSManager, entity::EntityKey,
};

use super::SystemSettings;

pub(crate) type Event<World> = Option<fn(&mut World, EventKey)>;

// A system that updates specific components in parallel
pub struct System<World> {
    pub(crate) cbitfield: Bitfield<u32>,
    // Events
    pub(crate) evn_run: Event<World>,
    pub(crate) evn_added_entity: Event<World>,
    pub(crate) evn_removed_entity: Event<World>,

    linked_components: RefCell<AHashMap<EntityKey, LinkedComponents>>,
    // Added, Removed
    added: RefCell<AHashMap<EntityKey, LinkedComponents>>,
    removed: RefCell<AHashMap<EntityKey, LinkedComponents>>,
}

impl<World> Default for System<World> {
    fn default() -> Self {
        Self {
            cbitfield: Default::default(),
            evn_run: Default::default(),
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
    pub(crate) fn add_entity(&self, key: EntityKey, linked_components: LinkedComponents) {
        let cloned = LinkedComponents {
            components: linked_components.components.clone(),
            mutated_components: linked_components.mutated_components.clone(),
            linked: linked_components.linked.clone(),
            key,
        };
        let mut lock = self.linked_components.borrow_mut();
        lock.insert(key, linked_components);
        let mut lock = self.added.borrow_mut();
        lock.insert(key, cloned);
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&self, key: EntityKey, linked_components: LinkedComponents) {
        let mut lock = self.linked_components.borrow_mut();
        if lock.contains_key(&key) {
            lock.remove(&key);
            let mut removed_lock = self.removed.borrow_mut();
            removed_lock.insert(key, linked_components);
        }
    }
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system(&self, world: &mut World, settings: SystemSettings) {
        // Create the component queries
        let linked_components = self.evn_run.map(|_| self.linked_components.borrow_mut());

        // Get the added components
        let mut borrowed = self.added.borrow_mut();
        let taken = std::mem::take(&mut *borrowed);
        // I hate this
        let rc = Rc::new(RefCell::new(taken));
        let added_components = self.evn_added_entity.map(|_| rc.borrow_mut());

        // Do a bit of decrementing
        let removed = self.removed.borrow_mut();
        let mut lock = settings.to_remove.borrow_mut();
        for (_, components) in lock.iter_mut() {
            if removed.contains_key(&components.key) {
                // Decrement
                components.counter -= 1;
            }
        }
        // Trolling purposes
        let mut borrowed = removed;
        let taken = std::mem::take(&mut *borrowed);
        // I hate this
        let rc = Rc::new(RefCell::new(taken));
        let removed_components = self.evn_removed_entity.map(|_| rc.borrow_mut());

        // Run the "Added Entity" and "Removed Entity" events first, then we can run the "Run System" event
        if let Some(evn_added_entity) = self.evn_added_entity {
            evn_added_entity(world, EventKey::Query(ComponentQuery { linked_components: added_components }));
        }
        if let Some(evn_removed_entity) = self.evn_removed_entity {
            evn_removed_entity(world, EventKey::Query(ComponentQuery { linked_components: removed_components }));
        }
        if let Some(run_system_evn) = self.evn_run {
            run_system_evn(world, EventKey::Query(ComponentQuery { linked_components }));
        }
    }
}
