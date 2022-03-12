use std::cell::{RefCell, RefMut};

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{ComponentQuery, ComponentQuerySet, LinkedComponents, LinkedComponentsDelta},
    entity::EntityKey,
};

use super::{SubSystem, SystemSettings};
pub(crate) type Event<World> = Option<fn(&mut World, ComponentQuerySet)>;
// A system that contains multiple subsystems, each with their own component queries
pub struct System<World> {
    // Subsystems
    pub(crate) subsystems: Vec<(SubSystem, Bitfield<u32>)>,
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
            for (subsystem, _) in self.subsystems.iter() {
                let borrowed = subsystem;
                if borrowed.delta.borrow().removed.contains_key(&components.key) {
                    // Decrement
                    components.counter -= 1;
                }
            }
        }

        // The code trolled me on the March 7, 2022, at 7:43pm
        drop(lock);

        // Get the deltas
        let mut deltas = {
            self.subsystems.iter().map(|(subsystem, _)| {
                let mut delta = subsystem.delta.borrow_mut();
                let added = std::mem::take(&mut delta.added);
                let removed = std::mem::take(&mut delta.removed);
                // Apply the removal deltas as soon as possible
                let mut all = subsystem.all.borrow_mut();
                for (key, _) in removed.iter() {
                    all.remove(key);
                }
                LinkedComponentsDelta {
                    added,
                    removed,
                }
            }).collect::<Vec<_>>()
        };

        // Run
        if let Some(run_system_evn) = self.evn_run {
            // Get the queries (added, removed, all)
            let queries = self
                .subsystems
                .iter()
                .zip(deltas.iter_mut())
                .map(|((subsystem, _), deltas)| {
                    // Splitting
                    let borrowed = subsystem;
                    ComponentQuery { all: borrowed.all.borrow_mut(), delta: deltas }
                })
                .collect::<Vec<_>>();
            run_system_evn(world, queries);
        }

        // Apply the addition deltas
        for (delta, (subsystem, _)) in deltas.into_iter().zip(self.subsystems.iter()) {
            let mut all = subsystem.all.borrow_mut();
            // Add
            for (key, components) in delta.added {
                all.insert(key, components);
            }
        }
    }
}
