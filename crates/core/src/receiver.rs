use crate::{task::WorldTask, WorldTaskBatch, World, MutContext};


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
        Self {
            rx,
            batch_tasks: Vec::new(),
        }
    }
    // This will return true to each task batch that can be run currently
    fn filter_task_batches(task_batch: &WorldTaskBatch) -> bool {
        match task_batch.timing {
            // TODO: Actually program this properly
            crate::WorldTaskTiming::Earliest => true,
            crate::WorldTaskTiming::ByNextFrame => true,
            crate::WorldTaskTiming::Free => true,
        }
    }
    // Execute a single task
    fn execute(&mut self, mut_context: &MutContext, task: WorldTask) {
        // We will execute the tasks
        match task {
            WorldTask::AddEntity(entity, id, group) => {
                // We will add the entity to the world
                mut_context.ecs.add_entity(entity, id, group);
            },
            WorldTask::RemoveEntity(id) => {
                // We will remove the entity from the world
                mut_context.ecs.remove_entity(id).unwrap();
            },
            WorldTask::DirectAddComponent(_, _) => todo!(),
        }
    }
    // We will flush the tasks, and execute them
    // This is called at the end of each system execution, since some tasks might need to execute earlier than others
    pub fn flush(&mut self, mut_context: &MutContext) {
        self.batch_tasks.extend(self.rx.try_iter());
        let taken = self.batch_tasks.drain_filter(|x| Self::filter_task_batches(x)).collect::<Vec<_>>();
        for batch in taken {
            match batch.combination {
                crate::WorldTaskCombination::Batch(tasks) => for task in tasks { self.execute(mut_context, task) },
                crate::WorldTaskCombination::Single(task) => self.execute(mut_context, task),
            }
        }
    }
    // Execute immediate
    pub fn flush_immediate(&self, mut_context: &MutContext) {
        let batch_tasks = self.rx.try_iter().collect::<Vec<_>>();
        for batch in batch_tasks {
            match batch.combination {
                crate::WorldTaskCombination::Batch(tasks) => for task in tasks { self.execute(mut_context, task) },
                crate::WorldTaskCombination::Single(task) => self.execute(mut_context, task),
            }
        }
    }
}