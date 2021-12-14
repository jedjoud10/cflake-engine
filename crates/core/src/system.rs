use crate::communication::{WorldTaskSender, RECEIVER};
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::{collections::HashMap, sync::Mutex};

// Some special worker thread commands
pub enum WorkerThreadCommand {
    StopSystem, // Completely stop the system before we join the SystemWorkerThread
}

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

lazy_static! {
    // The number of systems
    pub static ref SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

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
    let system_id = SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let builder = std::thread::Builder::new().name(format!("SystemWorkerThread '{}'", system_id));
    let handler = builder
        .spawn(move || {
            // We must initialize the channels
            crate::command::initialize_channels_worker_thread();
            // Set our render command sender as well
            rendering::pipec::initialize_threadlocal_render_comms();
            // Create the system on this thread
            SENDER.with(|x| {
                let system = callback();
                let sender_ = x.borrow();
                let sender = sender_.as_ref().unwrap();
                let wtc_rx = &sender.wtc_rx;
                println!("Hello from '{}'!", std::thread::current().name().unwrap());
                // Start the system loop
                loop {
                    // Check if we have any system commands that must be executed
                    match wtc_rx.try_recv() {
                        Ok(wtc) => {
                            // Execute the worker thread command
                            match wtc {
                                WorkerThreadCommand::StopSystem => {
                                    println!("Shutting down '{}'...", std::thread::current().name().unwrap());
                                    break;
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    // Check the rendering callback buffer
                    rendering::pipeline::interface::fetch_threadlocal_callbacks();

                    // Start of the independent system frame
                    // End of the independent system frame, we must wait until the main thread allows us to continue
                    // Check if the system is still running
                    println!("Update system");
                    crate::global::main::thread_sync();
                }
                println!("Loop for '{}' has stopped!", std::thread::current().name().unwrap());
            });
        })
        .unwrap();
    // Add the tx
    let mut receiver_ = RECEIVER.lock().unwrap();
    let receiver = receiver_.as_mut().unwrap();
    receiver.wtc_txs.insert(handler.thread().id(), receiver.template_wtc_tx.clone());
    handler
}
