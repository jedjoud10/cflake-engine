use crate::{RenderCommand, RenderTask, RenderTaskReturn, RenderTaskStatus, basics::*};
use std::sync::mpsc::{Receiver, Sender};

// Render pipeline. Contains everything related to rendering. This is also ran on a separate thread
#[derive(Default)]
pub struct RenderPipeline {
    // Command ID
    pub command_id: u128,
    // The tasks that are asynchronous and are pending their return values
    pub pending_wait_list: Vec<(RenderCommand, Box<dyn FnMut(RenderTaskReturn)>)>,
    // TX and RX
    pub channel: Option<(Sender<RenderCommand>, Receiver<RenderCommand>)>,
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
        self.channel = Some((tx, rx));
    }

    // Complete a task immediatly
    pub fn task_immediate(&mut self, task: RenderTask) -> RenderTaskReturn {
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
            status: RenderTaskStatus::PendingStartup,
        };
        // Increment
        self.command_id += 1;
        // Send the command to the render thread
        match self.channel {
            Some((tx, rx)) => {
                tx.send(render_command);
                // Wait for the result
                let recv = rx.recv().unwrap();
                let output = RenderTaskReturn::Object(GPUObject::None);
                return output;
            },
            None => todo!(),
        }        
    }
    // Complete a task, but the result is not needed immediatly, and call the call back when the task finishes
    pub fn task<F>(&mut self, task: RenderTask, mut callback: F) where F: FnMut(RenderTaskReturn) + 'static {
        let boxed_fn_mut: Box<dyn FnMut(RenderTaskReturn)> = Box::new(callback); 
        // Create a new render command and send it to the separate thread
        let render_command = RenderCommand {
            message_id: self.command_id,
            input_task: task,
            status: RenderTaskStatus::PendingStartup,
        };
        // Increment
        self.command_id += 1;
        // Send the command to the render thread
        match self.channel {
            Some((tx, rx)) => {
                tx.send(render_command);
                // This time, we must add this to the wait list
                self.pending_wait_list.push((
                    render_command,
                    boxed_fn_mut
                ));
            },
            None => todo!(),
        }  
    }
}