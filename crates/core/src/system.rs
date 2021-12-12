use crate::communication::WorldTaskSender;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::thread::JoinHandle;
use std::{collections::HashMap, sync::Mutex};

// Some special worker thread commands
pub enum WorkerThreadCommand {}

lazy_static! {
    // The sender end of the worker thread commands
    pub static ref WTCOMMAND_SENDER: Mutex<WorkerThreadCommandSender> = Mutex::new(WorkerThreadCommandSender::default());
}

// WorkerThreadCommand sender
#[derive(Default)]
pub struct WorkerThreadCommandSender {
    pub wtc_txs: Option<HashMap<std::thread::ThreadId, crossbeam_channel::Sender<WorkerThreadCommand>>>,
}
// System command receiver
#[derive(Default)]
pub struct WorkerThreadsReceiver {}

// The system group thread data is local to each system thread
thread_local! {
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
    // The receiving end of the system commands
    pub static WORKER_THREADS_RECEIVER: RefCell<WorkerThreadsReceiver> = RefCell::new(WorkerThreadsReceiver::default());
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub static SENDER: RefCell<Option<WorldTaskSender>> = RefCell::new(None);
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData>(callback: F) -> JoinHandle<()>
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
{
    std::thread::spawn(move || {
        // We must initialize the channels
        crate::command::initialize_channels_worker_thread();
        // Create the system on this thread
        SENDER.with(|x| {
            let system = callback();
            let mut running = true;
            let sender_ = x.borrow();
            let sender = sender_.as_ref().unwrap();
            let wtc_rx = &sender.wtc_rx;

            // Start the system loop
            while running {
                // Start of the independent system frame
                // End of the independent system frame, we must wait until the main thread allows us to continue
                // Check if the system is still running
            }
        });
    })
}
