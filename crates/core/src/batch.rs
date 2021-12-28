use crate::{
    command::{CommandQuery, CommandQueryResult, CommandQueryType},
    tasks::Task,
};
use std::collections::HashMap;

// A thread local batch manager that is stored on each system worker thread
#[derive(Default)]
pub struct BatchManager {
    pub batches: HashMap<u32, BatchCommandQuery>,
}

// A batch of multiple world commands that will be sent to the world all at the same time
#[derive(Default)]
pub struct BatchCommandQuery {
    pub id: u32,
    pub tasks: Vec<Task>,
}

impl BatchCommandQuery {
    // Send the batch to the main thread. This will clear all of the stored tasks in the batch
    pub fn send(&mut self) {
        let thread_id = std::thread::current().id();
        let taken = std::mem::take(&mut self.tasks);
        let commands = taken
            .into_iter()
            .map(|task| {
                let query = CommandQuery {
                    task,
                    thread_id,
                    callback_id: None,
                };
                query
            })
            .collect::<Vec<CommandQuery>>();
        // Send the commands
        crate::command::command(CommandQueryType::Batch(commands));
    }
    // Add a command to this batch, so we can send it when we will send this batch
    pub fn add(&mut self, mut command_result: CommandQueryResult) {
        // Send the command
        self.tasks.push(command_result.task.take().unwrap());
    }
}
