
// Tasks
mod tasks {
    // Some world tasks
    pub enum Task {
        // Components
        CreateComponent(),
        DestroyComponent(),
        // Entity
        CreateEntity(),
        DestroyEntity(),
        // Direct Linking
        LinkComponent(),
        UnlinkComponent(),
    }
    // And their corresponding output
    pub enum TaskReturn {
        // Entity
        CreateEntity(WaitableTask<usize>),
        DestroyEntity(WaitableTask<Option<()>>),
    }
    // The return type for a world task, we can wait for the return or just not care lol
    pub struct WaitableTask<T> {
        pub id: u64,
        pub val: T,
    }

    impl<T> WaitableTask<T> {
        // Wait for the main thread to finish this specific task
        pub fn wait(self) -> T {
            // Wait for the main thread to send back the confirmation
            todo!()
        }
    }
}
// Command groups
mod commands {
    // A sent command query
    pub enum CommandQuery {
        Group(Vec<Command>),
        Singular(Command),
    }
    // The return value for the command query
    pub enum CommandReturn {
        Group(Vec<TaskReturn>),
        Singular(TaskReturn)
    }
    pub use super::tasks::Task;
    use super::tasks::TaskReturn;
    // A world command
    pub struct Command {
       pub id: u64,
       pub task: Task,
    }
}


// Sending - Receiving
use std::sync::mpsc::{Sender, Receiver};
use self::commands::*;
// A task sender that will send tasks from other thread to the main thread, asynchronously.
pub struct WorldTaskSender {
    pub to_main_groups: Option<(Sender<CommandQuery>, Receiver<CommandQuery>)>, // SentCommands. WorkerThreads -> MainThread
    pub to_thread_group: Option<(Sender<CommandQuery>, Receiver<CommandQuery>)>, // SentCommandQuerry. MainThread -> WorkerThreads
}