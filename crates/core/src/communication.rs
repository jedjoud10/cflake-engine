use crate::{command::CommandQuery, tasks::WaitableTask};
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
    // A counter for the number of commands issued
    pub static ref COUNTER: AtomicU64 = AtomicU64::new(0);
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub  static ref SENDER: Mutex<WorldTaskSender> = Mutex::new(WorldTaskSender::default());
    // Receiver of tasks. Is called on the main thread, receives messages from the worker threads
    pub  static ref RECEIVER: Mutex<WorldTaskReceiver> = Mutex::new(WorldTaskReceiver::default());
}
// Worker threads
#[derive(Default)]
pub struct WorldTaskSender {
    pub tx: Option<Sender<(u64, CommandQuery)>>, // CommandQuery. WorkerThreads -> MainThread
}
// Main thread
#[derive(Default)]
pub struct WorldTaskReceiver {
    pub rx: Option<Receiver<(u64, CommandQuery)>>,                                            // CommandQuery. WorkerThreads -> MainThread
    pub txs: Option<HashMap<std::thread::ThreadId, crossbeam_channel::Sender<WaitableTask>>>, // WaitableTask. MainThread -> WorkerThreads
}
