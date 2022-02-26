use crate::object::PipelineTask;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::sync::mpsc::SendError;
use std::sync::{mpsc::Sender, Mutex};

use super::Pipeline;

// We will store a global sender, that way we can copy it to the other threads using an init coms method
lazy_static! {
    static ref SENDER: Mutex<Option<Sender<PipelineTask>>> = Mutex::new(None);
}

// Thread local sender
thread_local! {
    static LOCAL_SENDER: RefCell<Option<Sender<PipelineTask>>> = RefCell::new(None);
    static RENDER_THREAD: Cell<bool> = Cell::new(false);
}

// Check if we are on the render thread
pub fn on_render_thread() -> bool {
    RENDER_THREAD.with(|cell| cell.get())
}

// Set the global sender
pub(crate) fn set_global_sender(sender: Sender<PipelineTask>) {
    {
        let mut lock = SENDER.lock().unwrap();
        *lock = Some(sender);
    }
    // We cannot send tasks to the pipeline from the render thread itself
    RENDER_THREAD.with(|cell| cell.set(true));
}

// Send a task using the thread local sender
pub(crate) fn send_task(task: PipelineTask, pipeline: &Pipeline) -> Result<(), SendError<PipelineTask>> {
    // Set the local sender if it is still not valid
    if LOCAL_SENDER.with(|cell| cell.borrow().is_none()) {
        // Get the global sender and copy it to the local sender
        let lock = SENDER.lock().unwrap();
        let sender = (*lock).as_ref().unwrap();
        LOCAL_SENDER.with(|cell| {
            let mut cell = cell.borrow_mut();
            *cell = Some(sender.clone());
        })
    }
    // If we are on the render thread, add the task directly
    if on_render_thread() {
        pipeline.add_task_internally(task);
        Ok(())
    } else {
        LOCAL_SENDER.with(|cell| {
            let cell = cell.borrow();
            let sender = (*cell).as_ref().unwrap();
            sender.send(task)
        })
    }
}
