use crate::tasks::WaitableTask;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    // A special channel just for synchronizing the worker threads
    static ref SYNCHRONIZE: Mutex<WorkThreadSync> = Mutex::new(WorkThreadSync::default());
}

#[derive(Default)]
pub struct WorkThreadSync {}

// Some data for a system group thread
#[derive(Default)]
pub struct SystemGroupThreadData {
    pub buffer: HashMap<u64, WaitableTask>,                    // The receiving buffer
    pub rx: Option<crossbeam_channel::Receiver<WaitableTask>>, // The receiver
}
// The system group thread data is local to each system thread
thread_local! {
    pub static SYSTEM_GROUP_THREAD_DATA: RefCell<SystemGroupThreadData> = RefCell::new(SystemGroupThreadData::default());
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
}
