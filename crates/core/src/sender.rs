use crate::WorldTask;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::{mpsc::Sender, Mutex};

// We will store a global sender, that way we can copy it to the other threads using an init coms method
lazy_static! {
    static ref SENDER: Mutex<Option<Sender<WorldTask>>> = Mutex::new(None);
}

// Thread local sender
thread_local! {
    static LOCAL_SENDER: RefCell<Option<Sender<WorldTask>>> = RefCell::new(None);
    pub(crate) static INTERNAL_TASKS: RefCell<Vec<WorldTask>> = RefCell::new(Vec::new());
}

// Set the global sender
pub(crate) fn set_global_sender(sender: Sender<WorldTask>) {
    {
        let mut lock = SENDER.lock().unwrap();
        *lock = Some(sender);
    }
    {
        others::set_main_thread();
    }
}

// Send a task using the thread local sender
pub(crate) fn send_task(task: WorldTask) -> Option<()> {
    // Initialize the sender if it is not valid yet
    if LOCAL_SENDER.with(|cell| cell.borrow().is_none()) {
        // Get the global sender and copy it to the local sender
        let lock = SENDER.lock().unwrap();
        let sender = (*lock).as_ref().unwrap();
        LOCAL_SENDER.with(|cell| {
            let mut cell = cell.borrow_mut();
            *cell = Some(sender.clone());
        });
    }

    // If we are on the main thread, add the task internally
    if others::on_main_thread() {
        INTERNAL_TASKS.with(|x| {
            let mut internal_tasks = x.borrow_mut();
            internal_tasks.push(task);
        });
        return Some(());
    }
    LOCAL_SENDER.with(|cell| {
        let cell = cell.borrow();
        let sender = (*cell).as_ref()?;
        sender.send(task).ok()
    })
}
