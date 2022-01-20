use ecs::manager_special::{remove_entity, add_entity};

use crate::{task::WorldTask, World, WorldTaskBatch, INTERNAL_TASKS};

// A receiver that we can use to receive tasks from other threads
pub struct WorldTaskReceiver {
    // An internal receiver
    rx: std::sync::mpsc::Receiver<WorldTaskBatch>,
    // Buffered tasks that we can run at a later time
    batch_tasks: Vec<WorldTaskBatch>,
}

impl WorldTaskReceiver {
    // Create a new world task receiver
    // Also setup the global channel
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<WorldTaskBatch>();
        crate::sender::set_global_sender(tx);
        Self { rx, batch_tasks: Vec::new() }
    }
    // Execute a single task
    fn execute(&mut self, world: &mut World, task: WorldTask) {
        // We will execute the tasks
        let ecs = &mut world.ecs;
        match task {
            WorldTask::AddEntity(entity, id, group) => {
                // We will add the entity to the world
                add_entity(ecs, entity, id, group);
            }
            WorldTask::RemoveEntity(id) => {
                // We will remove the entity from the world
                remove_entity(ecs, id).unwrap();
            }
            WorldTask::DirectAddComponent(_, _) => todo!(),
        }
    }
    // We will flush the tasks, and execute them
    // This is called at the end of each system execution, since some tasks might need to execute earlier than others
    pub fn flush(&mut self, world: &mut World) {
        self.batch_tasks.extend(self.rx.try_iter());
        let mut taken = std::mem::take(&mut self.batch_tasks);
        // Also poll the tasks that we have stored internally
        let internal_tasks = INTERNAL_TASKS.with(|x| std::mem::take(&mut *x.borrow_mut()));
        taken.extend(internal_tasks);
        for batch in taken {
            match batch.combination {
                crate::WorldTaskCombination::Batch(tasks) => {
                    for task in tasks {
                        self.execute(world, task)
                    }
                }
                crate::WorldTaskCombination::Single(task) => self.execute(world, task),
            }
        }
    }
}
