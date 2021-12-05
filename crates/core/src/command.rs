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
    pub struct WaitableTask {
        pub id: u64,
        pub thread_id: std::thread::ThreadId,
    }    

    // Excecute a specific task and give back it's result
    pub fn excecute_task(t: Task) -> TaskReturn {
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
        Group(u64, std::thread::ThreadId, Vec<Task>),
        Singular(u64, std::thread::ThreadId, Task),
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
}
// Worker threads
#[derive(Default)]
pub struct WorldTaskSender {
    pub tx: Option<Sender<CommandQuery>>, // CommandQuery. WorkerThreads -> MainThread 
    pub rx: Option<crossbeam_channel::Receiver<WaitableTask>> // WaitableTask<TaskReturn>. MainThread -> WorkerThreads
}
// Main thread
#[derive(Default)]
pub struct WorldTaskReceiver {
    pub tx: Option<crossbeam_channel::Sender<WaitableTask>>, // WaitableTask<TaskReturn>. MainThread -> WorkerThreads
    pub rx: Option<Receiver<CommandQuery>>, // CommandQuery. WorkerThreads -> MainThread
}
impl WaitableTask {
    // Wait for the main thread to finish this specific task
    pub fn wait(self) -> TaskReturn {
        // Wait for the main thread to send back the return task
        let sender = SENDER.lock().unwrap();
        let rx = sender.rx.unwrap();
        rx.
    }
}
// The functions
pub fn initialize_channels() {
    // Create the channels
    let (tx, rx) = std::sync::mpsc::channel::<CommandQuery>();
    let (tx2, rx2) = crossbeam_channel::unbounded::<WaitableTask>();
    let receiver = RECEIVER.lock().unwrap();
    let sender = SENDER.lock().unwrap();
    // The task senders
    receiver.rx = Some(rx);
    sender.tx = Some(tx);
    // The taskreturn senders
    sender.rx = Some(rx2);
    receiver.tx = Some(tx2);
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread() {
    // Poll each command query
    let receiver = RECEIVER.lock().unwrap();
    let tx = receiver.tx.unwrap();
    for x in receiver.rx.unwrap().try_recv() {
        let (id, thread_id, task) = match x {
            CommandQuery::Group(id, thread_id, tgroup) => {
                // Execute the task group
                (id, thread_id, tasks::excecute_task(tgroup[0]))
            },
            CommandQuery::Singular(id, thread_id, t) => {
                // Execute the singular task        
                (id, thread_id, tasks::excecute_task(t))
            },
        };
        let waitabletask = WaitableTask { id, thread_id, val: Some(task) };
        // Send the result back to system threads
        tx.send(waitabletask).unwrap();
    }
}
// Send a command query to the world, giving back a command return that can be waited for
pub fn command(query: CommandQuery) -> WaitableTask {
    // Send the command query
    let x = SENDER.lock().unwrap();
    let tx = x.tx.as_ref().unwrap();
    tx.send(query);
    // Get the corresponding return command value
    match query {
        CommandQuery::Group(id, thread_id, tasks) => WaitableTask {
            id,
            thread_id: std::thread::current().id(),
        },
        CommandQuery::Singular(id, thread_id, task) => WaitableTask {
            id,
            thread_id: std::thread::current().id(),
        },
    }
}