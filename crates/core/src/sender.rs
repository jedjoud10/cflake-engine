use crate::WorldTaskBatch;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::{mpsc::Sender, Mutex};

// We will store a global sender, that way we can copy it to the other threads using an init coms method
lazy_static! {
    static ref SENDER: Mutex<Option<Sender<WorldTaskBatch>>> = Mutex::new(None);
}

// Thread local sender
thread_local! {
    static LOCAL_SENDER: RefCell<Option<Sender<WorldTaskBatch>>> = RefCell::new(None);
    pub(crate) static INTERNAL_TASKS: RefCell<Vec<WorldTaskBatch>> = RefCell::new(Vec::new());
    static MAIN_THREAD: RefCell<bool> = RefCell::new(false);
}

// Set the global sender
pub(crate) fn set_global_sender(sender: Sender<WorldTaskBatch>) {
    {
        let mut lock = SENDER.lock().unwrap();
        *lock = Some(sender);
    }
    {
        MAIN_THREAD.with(|cell| {
            let mut cell = cell.borrow_mut();
            *cell = true;
        });
    }
}

// Initialize the thread local sender. We must do this after the world has been fully initialized
pub fn init_coms() {
    // Get the global sender and copy it to the local sender
    let lock = SENDER.lock().unwrap();
    let sender = (&*lock).as_ref().unwrap();
    LOCAL_SENDER.with(|cell| {
        let mut cell = cell.borrow_mut();
        *cell = Some(sender.clone());
    })
}

// Send a task using the thread local sender
pub(crate) fn send_task(task_batch: WorldTaskBatch) -> Option<()> {
    // If we are on the main thread, add the task internally
    if MAIN_THREAD.with(|x| *x.borrow()) {
        INTERNAL_TASKS.with(|x| {
            let mut internal_tasks = x.borrow_mut();
            internal_tasks.push(task_batch);
        });
        return Some(());
    }
    LOCAL_SENDER.with(|cell| {
        let cell = cell.borrow();
        let sender = (&*cell).as_ref()?;
        sender.send(task_batch).ok()
    })
}
