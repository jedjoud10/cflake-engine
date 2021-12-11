// Tasks
mod tasks {
    use std::sync::mpsc::Receiver;

    // Some world tasks
    pub enum Task {
        // Entity
        EntityAdd(ecs::Entity, ecs::ComponentLinkingGroup),
        EntityRemove(usize),
        // This is only valid if the entity is also valid
        ComponentLinkDirect(usize, usize),
        ComponentUnlinkDirect(usize, usize),
        // UI
        AddRoot(String, ui::Root),
        SetRootVisibility(bool),
        // Main 
        CreateConfigFile(),
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
        pub val: Option<TaskReturn>,
        pub thread_id: std::thread::ThreadId,
    }    

    // Excecute a specific task and give back it's result
    pub fn excecute_task(t: Task, world: &mut crate::world::World) -> TaskReturn {
        match t {
            Task::EntityAdd(_, _) => todo!(),
            Task::EntityRemove(_) => todo!(),
            Task::ComponentLinkDirect(_, _) => todo!(),
            Task::ComponentUnlinkDirect(_, _) => todo!(),
            Task::AddRoot(_, _) => todo!(),
            Task::SetRootVisibility(_) => todo!(),
            Task::CreateConfigFile() => todo!(),
        }
    }
}
// Command groups
mod commands {
    // A sent command query
    pub struct CommandQuery {
        pub thread_id: std::thread::ThreadId,
        pub task: Task,
    }
    impl CommandQuery {
        // From single
        pub fn new(task: Task) -> Self {
            let thread_id = std::thread::current().id();
            Self { thread_id, task }
        }
    }
    pub use super::tasks::Task;
    use super::tasks::{TaskReturn, WaitableTask};
}

// Sending - Receiving
use std::{sync::{mpsc::{Sender, Receiver}, RwLock, Arc, Mutex, atomic::{Ordering, AtomicU64}}, collections::HashMap, cell::{Cell, RefCell}};
pub use self::commands::*;
pub use self::tasks::*;
use lazy_static::lazy_static;
lazy_static! {
    // A counter for the number of commands issued
    static ref COUNTER: AtomicU64 = AtomicU64::new(0);
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    static ref SENDER: Mutex<WorldTaskSender> = Mutex::new(WorldTaskSender::default());
    // Receiver of tasks. Is called on the main thread, receives messages from the worker threads
    static ref RECEIVER: Mutex<WorldTaskReceiver> = Mutex::new(WorldTaskReceiver::default());
}
// Some data for a system group thread
#[derive(Default)]
pub struct SystemGroupThreadData {
    pub buffer: HashMap<u64, WaitableTask>, // The receiving buffer
    pub rx: Option<crossbeam_channel::Receiver<WaitableTask>>, // The receiver
}
// The system group thread data is local to each system thread
thread_local! {
    static SYSTEM_GROUP_THREAD_DATA: RefCell<SystemGroupThreadData> = RefCell::new(SystemGroupThreadData::default());
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
}
// Worker threads
#[derive(Default)]
pub struct WorldTaskSender {
    pub tx: Option<Sender<(u64, CommandQuery)>>, // CommandQuery. WorkerThreads -> MainThread 
}
// Main thread
#[derive(Default)]
pub struct WorldTaskReceiver {
    pub rx: Option<Receiver<(u64, CommandQuery)>>, // CommandQuery. WorkerThreads -> MainThread
    pub txs: Option<HashMap<std::thread::ThreadId, crossbeam_channel::Sender<WaitableTask>>>, // WaitableTask. MainThread -> WorkerThreads
}
impl WaitableTask {
    // Wait for the main thread to finish this specific task
    pub fn wait(self) -> TaskReturn {
        /* #region We are already on the main thread */
        // No need to do any of this multithreading shit if we're already on the main thread
        let x = IS_MAIN_THREAD.with(|x| x.get());
        if x {
            return self.val.unwrap();
        }
        /* #endregion */
        /* #region Wait for the main thread to send a return task */
        // Wait for the main thread to send back the return task
        let sender = SENDER.lock().unwrap();
        let rx = SYSTEM_GROUP_THREAD_DATA.with(|x| {
            let system_group_thread_data = x.borrow();
            let rx = system_group_thread_data.rx.as_ref().unwrap().clone();
            rx
        });
        let thread_id = std::thread::current().id();
        let id = self.id;
        loop {
            // Receive infinitely until we get the valid return task value
            match rx.try_recv() {
                Ok(x) => {
                    // Either add this to the buffer and continue the loop or return early
                    if x.id == id {
                        // The same ID, we can exit early 
                        return x.val.unwrap();
                    } else {
                        // Add it to the buffer
                        let id = x.id;
                        SYSTEM_GROUP_THREAD_DATA.with(|data| {
                            let mut data = data.borrow_mut();
                            data.buffer.insert(id, x);
                        })
                    }
                },
                Err(_) => {
                    // Handle error
                },
            }
            let x: Option<TaskReturn> = SYSTEM_GROUP_THREAD_DATA.with(|data| {
                // Always check if the current group thread data contains our answer
                let mut data = data.borrow_mut();
                if data.buffer.contains_key(&id) {
                    // We found our answer!
                    let x = data.buffer.remove(&id).unwrap();
                    return Some(x.val.unwrap())
                } else {
                    None
                }
            });    
            return x.unwrap();        
        }
        /* #endregion */
    }
}
// The functions
pub fn initialize_channels() {
    // Create the channels
    let (tx, rx) = std::sync::mpsc::channel::<(u64, CommandQuery)>();
    let mut receiver = RECEIVER.lock().unwrap();
    let mut sender = SENDER.lock().unwrap();
    // The task senders
    receiver.txs = Some(HashMap::new());
    sender.tx = Some(tx);
    // The taskreturn senders
    receiver.rx = Some(rx);
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread() {
    // Poll each command query
    let receiver = RECEIVER.lock().unwrap();
    let rx = receiver.rx.as_ref().unwrap();
    let txs = receiver.txs.as_ref().unwrap();
    let mut world = crate::world::world_mut();
    for (id, query) in rx.try_recv() {
        let waitabletask = WaitableTask { 
            id, 
            thread_id: query.thread_id, 
            val: Some(excecute_task(query.task, &mut world))
        };
        // Send the result to the corresponding system threads
        match txs.get(&query.thread_id) {
            Some(x) => {
                // Send the return value to the corresponding receiver
                x.send(waitabletask).unwrap();
            },
            None => { /* Not the correct thread id */ },
        }
    }
}
// Send a command query to the world, giving back a command return that can be waited for
pub fn command(query: CommandQuery) -> WaitableTask {
    // Send the command query
    let x = SENDER.lock().unwrap();
    let tx = x.tx.as_ref().unwrap();
    let id = COUNTER.fetch_add(0, Ordering::Relaxed);
    let is_main_thread = IS_MAIN_THREAD.with(|x| x.get());
    // Early main thread exit lol
    if !is_main_thread {
        tx.send((id, query)).unwrap();    
        // Increment the counter
        // Get the corresponding return command value
        WaitableTask { 
            id,
            thread_id: std::thread::current().id(),
            val: None,
        }
    } else {
        // This is the main thread calling, we don't give a  f u c k   
        WaitableTask { 
            id,
            thread_id: std::thread::current().id(),
            val: {
                let mut world = crate::world::world_mut();
                let output = tasks::excecute_task(query.task, &mut world);
                Some(output)
            },
        }    
    }
}