use crate::tasks::WaitableTask;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::thread::JoinHandle;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    // A special channel just for synchronizing the worker threads
    static ref SYNCHRONIZE: Mutex<WorkThreadSync> = Mutex::new(WorkThreadSync::default());
}

#[derive(Default)]
pub struct WorkThreadSync {}

// Some data for a system group thread
#[derive(Default)]
pub struct WorkerThreadCommonData {
    pub buffer: HashMap<u64, WaitableTask>,                    // The receiving buffer
    pub rx: Option<crossbeam_channel::Receiver<WaitableTask>>, // The receiver
}
// The system group thread data is local to each system thread
thread_local! {
    pub static SYSTEM_GROUP_THREAD_DATA: RefCell<WorkerThreadCommonData> = RefCell::new(WorkerThreadCommonData::default());
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData>(callback: F) -> JoinHandle<()>
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
{
    std::thread::spawn(move || {
        // Create the system on this thread
        let system = callback();
        let mut running = true;
        let rx = crate::communication::SENDER.lock()

        // Start the system loop
        while running {
            // Start of the independent system frame
            // End of the independent system frame, we must wait until the main thread allows us to continue
            // Check if the system is still running
        }
    })
}