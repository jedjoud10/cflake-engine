use std::sync::mpsc::SendError;
use std::sync::{mpsc::Sender, Mutex};
use std::cell::RefCell;
use lazy_static::lazy_static;

use crate::object::{PipelineTask, TaskID};

// We will store a global sender, that way we can copy it to the other threads using an init coms method
lazy_static! {
    static ref SENDER: Mutex<Option<Sender<(PipelineTask, TaskID)>>> = Mutex::new(None);
}

// Thread local sender
thread_local! {
    static LOCAL_SENDER: RefCell<Option<Sender<(PipelineTask,TaskID)>>> = RefCell::new(None);
}

// Set the global sender
pub(crate) fn set_global_sender(sender: Sender<(PipelineTask,TaskID)>) {
    {
        let mut lock = SENDER.lock().unwrap();
        *lock = Some(sender);
    }
    // Also set the render's thread thread local sender
    init_coms();
}

// Initialize the thread local sender. We must do this after the render pipeline has been fully initialized
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
pub fn send_task(task: (PipelineTask, TaskID)) -> Result<(), SendError<(PipelineTask, TaskID)>> {
    LOCAL_SENDER.with(|cell| {
        let cell = cell.borrow();
        let sender = (&*cell).as_ref().unwrap();
        sender.send(task)
    })
} 