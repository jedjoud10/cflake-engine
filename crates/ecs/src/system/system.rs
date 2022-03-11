use std::{cell::{RefCell, RefMut}};

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{LinkedComponents, ComponentQuerySet},
    entity::EntityKey,
    event::EventKey,
};

use super::{SystemSettings, SubSystem};
pub(crate) type Event<World> = Option<fn(&mut World, EventKey)>;
// A system that contains multiple subsystems, each with their own component queries
pub struct System<World> {
    // Subsystems
    pub(crate) subsystems: Vec<RefCell<SubSystem>>,
    // Events
    pub(crate) evn_run: Event<World>,
    pub(crate) evn_added_entity: Event<World>,
    pub(crate) evn_removed_entity: Event<World>,
}

impl<World> Default for System<World> {
    fn default() -> Self {
        Self {
            subsystems: Default::default(),
            evn_run: Default::default(),
            evn_added_entity: Default::default(),
            evn_removed_entity: Default::default(),
        }
    }
}

// System code
impl<World> System<World> {
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system(&self, world: &mut World, settings: SystemSettings) {
        // Do a bit of decrementing
        let mut lock = settings.to_remove.borrow_mut();
        for (_, components) in lock.iter_mut() {
            // Check subsystems
            self.subsystems.iter().map(|x| x.borrow()).for_each(|mut subsystems| 
                if subsystems.removed.contains_key(&components.key) {
                    // Decrement
                    components.counter -= 1;
                }
            );
        }

        // The code trolled me on the March 7, 2022, at 7:43pm
        drop(lock);

        // Run the "Added Entity" and "Removed Entity" events first, then we can run the "Run System" event
        if let Some(evn_added_entity) = self.evn_added_entity {
            // Get the queries
            let queries = self.subsystems.iter().map(|x| Some(RefMut::map(x.borrow_mut(), |subsystem| &mut subsystem.added))).collect::<Vec<_>>();
            evn_added_entity(
                world,
                EventKey::Queries(ComponentQuerySet {
                    queries: queries,
                }),
            );
        }
        /*
        if let Some(evn_removed_entity) = self.evn_removed_entity {
            evn_removed_entity(
                world,
                EventKey::Queries({                    
                    // TODO: Fix this as well
                    let mut vec = Vec::new();
                    for mut removed in removed_components.into_iter() {
                        vec.push(ComponentQuery {
                            linked_components: RefMut::map(removed, |x| x),
                        })
                    }
                    vec
                }),
            );
        }
        if let Some(run_system_evn) = self.evn_run {
            // If we don't have any components, we can still execute the event
            run_system_evn(
                world,
                EventKey::Queries(linked_components.into_iter().map(|mut  a| ComponentQuery { linked_components: &mut a }).collect::<Vec<_>>())
            );
        }

        // Clear at the end (I use clear and not std::mem::take because it would avoid making more heap allocations in the future)
        for subsystem in self.subsystems.iter() {
            let mut added = subsystem.added.borrow_mut();
            added.clear();
            let mut removed = subsystem.removed.borrow_mut();
            removed.clear();
        }
        */
    }
}
