// Tasks
mod tasks {
    // Some world tasks
    pub enum Task {
        // Components
        CreateComponentDirect(),
        DestroyComponentDirect(usize),
        // Entity
        CreateEntity(ecs::Entity, ecs::ComponentLinkingGroup),
        DestroyEntity(),
        // This is only valid if the entity is also valid
        LinkComponentDirect(usize, usize),
        UnlinkComponentDirect(usize, usize),
    }
    // And their corresponding output
    pub enum TaskReturn {
        // Entity
        CreateEntity(usize),
        DestroyEntity(Option<()>),
    }
    // The return type for a world task, we can wait for the return or just not care lol
    pub struct WaitableTask<T> {
        pub id: u64,
        pub val: Option<T>,
    }

    impl<T> WaitableTask<T> {
        // Wait for the main thread to finish this specific task
        pub fn wait(self) -> T {
            // Wait for the main thread to send back the confirmation
            todo!()
        }
        // Use a callback instead of waiting
        pub fn callback<F>(self, callback: F) where F: Fn(T) + Send + 'static {
            others::callbacks::add_callback(callback);
        }
    }

    // Excecute a specific task and give back it's result
    fn excecute_task(t: Task) -> WaitableTask<TaskReturn> {
        match t {
            Task::CreateComponentDirect() => todo!(),
            Task::DestroyComponentDirect(_) => todo!(),
            Task::CreateEntity(_, _) => todo!(),
            Task::DestroyEntity() => todo!(),
            Task::LinkComponentDirect(_, _) => todo!(),
            Task::UnlinkComponentDirect(_, _) => todo!(),
        }
    }
}
// Command groups
mod commands {
    // A sent command query
    pub enum CommandQuery {
        Group(u64, Vec<Task>),
        Singular(u64, Task),
    }
    pub use super::tasks::Task;
    use super::tasks::{TaskReturn, WaitableTask};
}

// Sending - Receiving
use std::sync::{mpsc::{Sender, Receiver}, RwLock, Arc, Mutex};
pub use self::commands::*;
pub use self::tasks::*;
use lazy_static::lazy_static;
lazy_static! {
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    static ref SENDER: Mutex<WorldTaskSender> = Mutex::new(WorldTaskSender::default());
    // Receiver of tasks. Is called on the main thread, receives messages from the worker threads
    static ref RECEIVER: Mutex<WorldTaskReceiver> = Mutex::new(WorldTaskReceiver::default());
    // The waitable tasks' return values that would be polled by the worker threads when they run WaitableTask.wait()
    static ref PENDING_RETURNS: Arc<Mutex<Vec<WaitableTask<TaskReturn>>>> = Arc::new(Mutex::new(Vec::new()));
}
// Worker threads
#[derive(Default)]
pub struct WorldTaskSender {
    pub tx: Option<Sender<CommandQuery>>, // CommandQuery. WorkerThreads -> MainThread 
}
// Main thread
#[derive(Default)]
pub struct WorldTaskReceiver {
    pub rx: Option<Receiver<CommandQuery>>, // CommandQuery. WorkerThreads -> MainThread
}
// The functions
pub fn initialize_channels() {
    // Create the channels
    let (tx, rx) = std::sync::mpsc::channel::<CommandQuery>();
    let receiver = RECEIVER.lock().unwrap();
    let sender = SENDER.lock().unwrap();
    let mut pending_returns = PENDING_RETURNS.as_ref().lock().unwrap();
    *pending_returns = Vec::new();
    receiver.rx = Some(rx);
    sender.tx = Some(tx);
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread() {
    // Poll each command query
    let receiver = RECEIVER.lock().unwrap();
    for x in receiver.rx.unwrap().try_recv() {
        match x {
            CommandQuery::Group(id, tgroup) => {
                // Execute the task group
            },
            CommandQuery::Singular(id, t) => {
                // Execute the singular task                
            },
        }
    }
}
// Send a command query to the world, giving back a command return that can be waited for
pub fn command<T>(query: CommandQuery) -> WaitableTask<T> {
    // Send the command query
    let x = SENDER.lock().unwrap();
    let tx = x.tx.as_ref().unwrap();
    tx.send(query);
    // Get the corresponding return command value
    match query {
        CommandQuery::Group(id, commands) => WaitableTask {
            id,
            val: None,
        },
        CommandQuery::Singular(id, command) => WaitableTask {
            id,
            val: None,
        },
    }
}
// Send multiple tasks