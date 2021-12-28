use std::collections::HashMap;
use crate::command::{CommandQueryResult, CommandQuery, CommandQueryType};

// A thread local batch manager that is stored on each system worker thread
#[derive(Default)]
pub struct BatchManager {
    pub batches: HashMap<u32, BatchCommandQueryResult>
}

pub struct BatchCommandQuery {
    pub commands: Vec<CommandQuery>,
}

// A batch of multiple world commands that will be sent to the world all at the same time
pub struct BatchCommandQueryResult {
    pub id: u32,
    commands: Vec<CommandQueryResult>
}

impl BatchCommandQueryResult {
    // New
    pub fn new(id: u32, commands: Vec<CommandQueryResult>) -> Self {
        Self {
            id,
            commands,
        }
    }
    // Send the batch to the main thread
    pub fn send(self) {
        let thread_id = std::thread::current().id();
        let commands = self.commands.into_iter().map(|mut res | {
            // Send the command
            let task = res.task.take().unwrap();
            let query = CommandQuery {
                task,
                thread_id,
                callback_id: None,
            };
            query
        }).collect::<Vec<CommandQuery>>();
        let batch = BatchCommandQuery { commands };
        crate::command::command(CommandQueryType::Batch(batch));
    }
}