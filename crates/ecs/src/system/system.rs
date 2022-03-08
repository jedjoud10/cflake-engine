use std::cell::RefCell;

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{ComponentQuery, LinkedComponents},
    entity::EntityKey,
    event::EventKey,
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
        let mut borrowed_added_components = self.added.borrow_mut();
        let added_components = self.evn_added_entity.map(|_| &mut borrowed_added_components);

        // Do a bit of decrementing
        let removed = self.removed.borrow_mut();
        let mut lock = settings.to_remove.borrow_mut();
        for (_, components) in lock.iter_mut() {
            if removed.contains_key(&components.key) {
                // Decrement
                components.counter -= 1;
            }
        }

        // The code trolled me on the March 7, 2022, at 7:43pm
        drop(lock);

        // Trolling purposes
        let mut borrowed_removed_components = removed;
        let removed_components = self.evn_removed_entity.map(|_| &mut borrowed_removed_components);

        // Run the "Added Entity" and "Removed Entity" events first, then we can run the "Run System" event
        if let (Some(evn_added_entity), Some(added_components)) = (self.evn_added_entity, added_components) {
            evn_added_entity(
                world,
                EventKey::Query(ComponentQuery {
                    linked_components: added_components,
                }),
            );
        }
        if let (Some(evn_removed_entity), Some(removed_components)) = (self.evn_removed_entity, removed_components) {
            evn_removed_entity(
                world,
                EventKey::Query(ComponentQuery {
                    linked_components: removed_components,
                }),
            );
        }
        if let Some(run_system_evn) = self.evn_run {
            // If we don't have any components, we can still execute the event
            let mut default = AHashMap::<EntityKey, LinkedComponents>::default();
            if let Some(mut linked_components) = linked_components {
                run_system_evn(
                    world,
                    EventKey::Query(ComponentQuery {
                        linked_components: &mut linked_components,
                    }),
                );
            } else {
                run_system_evn(world, EventKey::Query(ComponentQuery { linked_components: &mut default }));
            }
        }

        // Clear at the end (I use clear and not std::mem::take because it would avoid making more heap allocations in the future)
        borrowed_added_components.clear();
        borrowed_removed_components.clear();
    }
}
