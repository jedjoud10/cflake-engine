use crate::{RenderCommand, RenderTask, RenderTaskReturn, RenderTaskStatus, basics::*};
use std::sync::mpsc::{Receiver, Sender};

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
#[derive(Default)]
pub struct RenderPipeline {
    // The pending tasks that must be completed
    pub pending_tasks: Vec<RenderTask>,
    // The tasks that are asynchronous and are pending their return values
    pub pending_return_tasks_ids: Vec<usize>,
    // TX and RX
    pub tx: Option<Sender<RenderCommand>>,
    pub rx: Option<Receiver<RenderCommand>>,
}

impl RenderPipeline {
    // The render thread that is continuously being ran
    fn frame(tx: Sender<RenderCommand>, rx: Receiver<RenderCommand>) {

    }
    // Create the new render thread
    pub fn initialize_render_thread(&mut self) {
        let (tx, rx): (Sender<RenderCommand>, Receiver<RenderCommand>) = std::sync::mpsc::channel();
        let x = std::thread::spawn(move || {
            // We must render every frame
            loop {
                Self::frame(tx, rx)
            }
        });
        // Vars
        self.tx = Some(tx);
        self.rx = Some(rx);
    }

    // Complete a task immediatly
    pub fn task_immediate(&mut self, task: RenderTask) -> RenderTaskReturn {
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.pending_tasks.len(),
            input_task: task,
            status: RenderTaskStatus::PendingStartup,
        };
        // Wait for the result
    }
    // Complete a task, but the result is not needed immediatly, and call the call back when the task finishes
    pub fn task<F>(&mut self, task: RenderTask, callback: F) where F: FnMut() {

    }
}
