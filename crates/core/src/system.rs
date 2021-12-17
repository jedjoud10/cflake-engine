use crate::communication::{WorldTaskSender, RECEIVER};
use crate::global::callbacks::LogicSystemCallbackResultData;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::{collections::HashMap, sync::Mutex};

// Some special system commands
pub enum LogicSystemCommand {
    RunCallback(u64, LogicSystemCallbackResultData)
}

lazy_static! {
    // The sender end of the logic system commands
    static ref LOGIC_SYSTEMS_COMMAND_SENDER: Mutex<LogicSystemCommandSender> = Mutex::new(LogicSystemCommandSender::default());
    // The number of systems
    pub static ref SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

// Command sender
#[derive(Default)]
pub struct LogicSystemCommandSender {
    pub lsc_txs: Option<HashMap<std::thread::ThreadId, crossbeam_channel::Sender<LogicSystemCommand>>>,
}
// Command receiver
#[derive(Default)]
pub struct LogicSystemCommandReceiver {}

// The system group thread data is local to each system thread
thread_local! {
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
    // The receiving end of the system commands
    pub static WORKER_THREADS_RECEIVER: RefCell<LogicSystemCommandReceiver> = RefCell::new(LogicSystemCommandReceiver::default());
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub static SENDER: RefCell<Option<WorldTaskSender>> = RefCell::new(None);
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData>(callback: F) -> JoinHandle<()>
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
{
    let system_id = SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let builder = std::thread::Builder::new().name(format!("LogicSystemThread '{}'", system_id));
    let barrier_data_ = crate::global::main::clone();    
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
                let lsc_rx = &sender.lsc_rx;
                println!("Hello from '{}'!", std::thread::current().name().unwrap());
                let barrier_data = barrier_data_.clone();
                // Start the system loop
                loop {
                    // Check if we have any system commands that must be executed
                    match lsc_rx.try_recv() {
                        Ok(lsc) => {
                            // Execute the logic system command
                            match lsc {
                                LogicSystemCommand::RunCallback(id, result_data) => crate::callbacks::execute_callback(id, result_data),
                            }
                        }
                        Err(_) => {}
                    }
                    // Check the rendering callback buffer
                    rendering::pipeline::interface::fetch_threadlocal_callbacks();

                    // Start of the independent system frame
                    // End of the independent system frame, we must wait until the main thread allows us to continue
                    // Check if the system is still running
                    //std::thread::sleep(std::time::Duration::from_millis(400));
                    if barrier_data.is_world_valid() {
                        barrier_data.thread_sync();
                        if barrier_data.is_world_destroyed() {
                            barrier_data.thread_sync_quit();
                            break;
                        }
                    }

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

// Send a LogicSystemCommand to a specific thread
pub fn send_lsc(lgc: LogicSystemCommand, thread_id: &std::thread::ThreadId) {
    // Get the sender
    let mut senders_ = LOGIC_SYSTEMS_COMMAND_SENDER.lock().unwrap();
    let senders = senders_.lsc_txs.as_ref().unwrap();
    let sender = senders.get(thread_id).unwrap();
    // Send the message
    sender.send(lgc).unwrap();
}