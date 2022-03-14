use crate::component::{ComponentQuery, ComponentQuerySet, LinkedComponents, LinkedComponentsDelta};

use super::{SubSystem, SystemSettings, SystemExecutionOrder};
pub(crate) type Event<World> = Option<fn(&mut World, ComponentQuerySet)>;
// A system that contains multiple subsystems, each with their own component queries
pub struct System<World> {
    pub(crate) subsystems: Vec<SubSystem>,
    pub(crate) evn_run: Event<World>,
    pub(crate) order: SystemExecutionOrder,
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
                if subsystem.delta.borrow().removed.contains_key(&components.key) {
                    // Decrement
                    components.counter -= 1;
                }
            }
        }

        // The code trolled me on the March 7, 2022, at 7:43pm
        drop(lock);

        // Get the deltas
        let mut deltas = {
            self.subsystems
                .iter()
                .map(|subsystem| {
                    let mut delta = subsystem.delta.borrow_mut();
                    let mut added = std::mem::take(&mut delta.added);
                    let removed = std::mem::take(&mut delta.removed);

                    // Apply the deltas as soon as possible
                    let mut all = subsystem.all.borrow_mut();

                    // Do this so we don't need to clone anything in the next step for unused entities
                    for (key, _) in removed.iter() {
                        added.remove(key);
                    }

                    // Add
                    for (key, components) in added.iter() {
                        all.insert(
                            *key,
                            LinkedComponents {
                                components: components.components.clone(),
                                mutated_components: components.mutated_components.clone(),
                                linked: components.linked.clone(),
                                key: components.key,
                            },
                        );
                    }

                    // Remove
                    for (key, _) in removed.iter() {
                        all.remove(key);
                        added.remove(key);
                    }

                    // Output
                    LinkedComponentsDelta { added, removed }
                })
                .collect::<Vec<_>>()
        };

        // Run
        if let Some(run_system_evn) = self.evn_run {
            // Get the queries (added, removed, all)
            let queries = self
                .subsystems
                .iter()
                .zip(deltas.iter_mut())
                .map(|(subsystem, delta)| ComponentQuery {
                    all: subsystem.all.borrow_mut(),
                    delta,
                })
                .collect::<Vec<_>>();
            run_system_evn(world, queries);
        }
    }
}
