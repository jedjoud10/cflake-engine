use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::mpsc::SendError;
use std::sync::{mpsc::Sender, Mutex};
use crate::global::WorldTask;

// We will store a global sender, that way we can copy it to the other threads using an init coms method
lazy_static! {
    static ref SENDER: Mutex<Option<Sender<WorldTask>>> = Mutex::new(None);
}

// Thread local sender
thread_local! {
    static LOCAL_SENDER: RefCell<Option<Sender<WorldTask>>> = RefCell::new(None);
}

// Set the global sender
pub(crate) fn set_global_sender(sender: Sender<WorldTask>) {
    {
        let mut lock = SENDER.lock().unwrap();
        *lock = Some(sender);
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
pub(crate) fn send_task(task: WorldTask) -> Option<()> {
    LOCAL_SENDER.with(|cell| {
        let cell = cell.borrow();
        let sender = (&*cell).as_ref()?;
        sender.send(task).ok()
    })
}
