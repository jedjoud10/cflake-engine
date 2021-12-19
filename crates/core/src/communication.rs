use crate::{command::CommandQuery, system::LogicSystemCommand};
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
    // A copy of the WorldTaskSender, because we will need to copy it for each worker thread
    pub static ref COMMUNICATION_CHANNEL_COPY: Mutex<Option<crate::communication::CommunicationChannelsCopied>> = Mutex::new(None);
}
// A copy of the communication channels for each worker thread
pub struct CommunicationChannelsCopied {
    pub tx: Sender<CommandQuery>,
    pub lsc_rx: crossbeam_channel::Receiver<LogicSystemCommand>,
}

// Some struct that sends tasks to the main thread. This is present on all the worker threads, since there is a 1 : n connection between the main thread and worker threads
pub struct WorldTaskSender {
    pub tx: Sender<CommandQuery>,                         // CommandQuery. WorkerThreads -> MainThread
    pub lsc_rx: crossbeam_channel::Receiver<LogicSystemCommand>, // WorkerThreadCommand. MainThread -> WorkerThreads
}
// Main thread
pub struct WorldTaskReceiver {
    pub rx: Receiver< CommandQuery>,                                                      // CommandQuery. WorkerThreads -> MainThread
    pub lsc_txs: HashMap<std::thread::ThreadId, crossbeam_channel::Sender<LogicSystemCommand>>, // WaitableTask. MainThread -> WorkerThreads
    // Some template values that will be copied
    pub template_wtc_tx: crossbeam_channel::Sender<LogicSystemCommand>,
}
