use crate::{command::{CommandQuery, CommandQueryType}, system::LogicSystemCommand};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::AtomicU64,
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

lazy_static! {
    // Receiver of tasks. Is called on the main thread, receives messages from the worker threads
    pub static ref RECEIVER: Mutex<Option<WorldTaskReceiver>> = Mutex::new(None);
}
// Main thread
pub struct WorldTaskReceiver {
    pub rx: Receiver<CommandQueryType>,                                          // CommandQuery. WorkerThreads -> MainThread
    pub lsc_txs: HashMap<std::thread::ThreadId, Sender<LogicSystemCommand>>, // WaitableTask. MainThread -> WorkerThreads
}
