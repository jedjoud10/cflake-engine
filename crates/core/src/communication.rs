use crate::{command::CommandQuery, system::WorkerThreadCommand};
use lazy_static::lazy_static;
use std::{
    cell::RefCell,
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
    // Receiver of tasks. Is called on the main thread, receives messages from the worker threads
    pub static ref RECEIVER: Mutex<Option<WorldTaskReceiver>> = Mutex::new(None);
    // A copy of the WorldTaskSender, because we will need to copy it for each worker thread
    pub static ref COMMUNICATION_CHANNEL_COPY: Mutex<Option<crate::communication::CommunicationChannelsCopied>> = Mutex::new(None);
}
// A copy of the communication channels for each worker thread
pub struct CommunicationChannelsCopied {
    pub tx: Sender<(u64, CommandQuery)>,
    pub wtc_rx: crossbeam_channel::Receiver<WorkerThreadCommand>,
}

// Some struct that sends tasks to the main thread. This is present on all the worker threads, since there is a 1 : n connection between the main thread and worker threads
pub struct WorldTaskSender {
    pub tx: Sender<(u64, CommandQuery)>,                          // CommandQuery. WorkerThreads -> MainThread
    pub wtc_rx: crossbeam_channel::Receiver<WorkerThreadCommand>, // WorkerThreadCommand. MainThread -> WorkerThreads
}
// Main thread
pub struct WorldTaskReceiver {
    pub rx: Receiver<(u64, CommandQuery)>,                                                       // CommandQuery. WorkerThreads -> MainThread
    pub wtc_txs: HashMap<std::thread::ThreadId, crossbeam_channel::Sender<WorkerThreadCommand>>, // WaitableTask. MainThread -> WorkerThreads
    // Some template values that will be copied
    pub template_wtc_tx: crossbeam_channel::Sender<WorkerThreadCommand>,
}
