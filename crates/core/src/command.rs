use crate::callbacks::CallbackType;
// Sending - Receiving
use crate::communication::*;
use crate::system::*;
use crate::tasks::*;
use std::{collections::HashMap, sync::atomic::Ordering};

// A sent command query
pub struct CommandQuery {
    pub thread_id: std::thread::ThreadId,
    pub callback_id: Option<u64>,
    pub task: Task,
}
impl CommandQuery {
    // From single
    pub fn new(task: Task, callback_id: Option<u64>) -> Self {
        let thread_id = std::thread::current().id();
        Self { thread_id, task, callback_id }
    }
}

// The immediate result for that command query
pub struct CommandQueryResult {
    task: Option<Task>,
}

trait Test<T>: Fn(&T) {}

impl CommandQueryResult {
    // Create a new query result from a specific command
    pub fn new(task: Task) -> Self {
        Self { task: Some(task) }
    }
    // Set callback for this specific command query result. It will receive a notif from the main thread when to execute this callback
    pub fn with_callback(mut self, callback_id: u64) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = CommandQuery::new(task, Some(callback_id));
        command(query);
    }
}

impl std::ops::Drop for CommandQueryResult {
    // Custom drop function that actually sends the command, just in case where we did not explicitly specified
    fn drop(&mut self) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = CommandQuery::new(task, None);
        command(query);
    }
}

// Initialize the main channels on the main thread
pub fn initialize_channels_main() {
    // Create the channels
    let (tx_command, rx_command) = std::sync::mpsc::channel::<(u64, CommandQuery)>();
    let (wtc_tx, wtc_rx) = crossbeam_channel::unbounded::<LogicSystemCommand>();
    let mut copy_ = COMMUNICATION_CHANNEL_COPY.lock().unwrap();
    let mut receiver_ = RECEIVER.lock().unwrap();
    // Set the main thread values
    *receiver_ = Some(WorldTaskReceiver {
        rx: rx_command,
        wtc_txs: HashMap::new(),
        template_wtc_tx: wtc_tx,
    });
    // And then the worker thread template values
    *copy_ = Some(CommunicationChannelsCopied { tx: tx_command, wtc_rx: wtc_rx });
    // This is indeed the main thread
    IS_MAIN_THREAD.with(|x| x.set(true));
    println!("Initialized the channels on the MainThread");
}
// Initialize the channels on a worker thread (This must be called on the worker thread)
pub fn initialize_channels_worker_thread() {
    crate::system::SENDER.with(|x| {
        let mut sender_ = x.borrow_mut();
        let sender = &mut *sender_;
        let copy_ = COMMUNICATION_CHANNEL_COPY.lock().unwrap();
        let copy = copy_.as_ref().unwrap();
        // We do the cloning
        *sender = Some(WorldTaskSender {
            tx: copy.tx.clone(),
            lsc_rx: copy.wtc_rx.clone(),
        });
    })
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread() {
    // Poll each command query
    let receiver_ = RECEIVER.lock().unwrap();
    let receiver = receiver_.as_ref().unwrap();
    let rx = &receiver.rx;
    let mut world = crate::world::world_mut();
    for (id, query) in rx.try_recv() {
        // Just execute the task
        excecute_query(query, &mut world);
    }
}
// Send a command query to the world, giving back a command return that can be waited for
fn command(query: CommandQuery) {
    println!("Calling thread {:?}", query.thread_id);
    // Increment the counter
    let id = COUNTER.fetch_add(0, Ordering::Relaxed);
    // Check if we are running on the main thread
    let is_main_thread = IS_MAIN_THREAD.with(|x| x.get());
    if is_main_thread {
        // This is the main thread calling, we don't give a  f u c k
        let mut world = crate::world::world_mut();
        // Execute the task on the main thread
        excecute_query(query, &mut world);
    } else {
        // Send the command query
        SENDER.with(|sender| {
            let sender_ = sender.borrow();
            let sender = sender_.as_ref().unwrap();
            let tx = &sender.tx;
            tx.send((id, query)).unwrap();
        })
    }
}
