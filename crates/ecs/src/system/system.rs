use std::cell::{RefCell, RefMut};

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{ComponentQuery, ComponentQuerySet, LinkedComponents},
    entity::EntityKey,
};

use super::{SubSystem, SystemSettings};
pub(crate) type Event<World> = Option<fn(&mut World, ComponentQuerySet)>;
// A system that contains multiple subsystems, each with their own component queries
pub struct System<World> {
    // Subsystems
    pub(crate) subsystems: Vec<RefCell<SubSystem>>,
    // Events
    pub(crate) evn_run: Event<World>,
}

impl<World> Default for System<World> {
    fn default() -> Self {
        Self {
            subsystems: Default::default(),
            evn_run: Default::default(),
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
            for subsystem in self.subsystems.iter() {
                let borrowed = subsystem.borrow();
                if borrowed.delta.removed.contains_key(&components.key) {
                    // Decrement
                    components.counter -= 1;
                }
            }
        }

        // The code trolled me on the March 7, 2022, at 7:43pm
        drop(lock);

        // Run
        if let Some(run_system_evn) = self.evn_run {
            // Get the queries (added, removed, all)
            let queries = self
                .subsystems
                .iter()
                .map(|x| {
                    // Splitting
                    let borrowed = x.borrow_mut();
                    let (delta, all) = RefMut::map_split(borrowed, |borrowed| (&mut borrowed.delta, &mut borrowed.all));
                    ComponentQuery { all, delta }
                })
                .collect::<Vec<_>>();
            run_system_evn(world, queries);

            // Clear
            for query in self.subsystems.iter() {
                let mut borrowed = query.borrow_mut();
                borrowed.delta.added.clear();
                borrowed.delta.removed.clear();
            }
        }
    }
}
