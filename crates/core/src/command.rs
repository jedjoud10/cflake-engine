use rendering::PipelineStartData;

use crate::batch::BatchCommandQuery;
// Sending - Receiving
use crate::communication::*;
use crate::global::callbacks::LogicSystemCallbackArguments;
use crate::system::*;
use crate::tasks::*;
use std::{collections::HashMap};

// This can either be a single task or a batch of tasks
pub enum CommandQueryType {
    Single(CommandQuery),
    Batch(BatchCommandQuery)
}

// A sent command query
pub struct CommandQuery {
    pub thread_id: std::thread::ThreadId,
    pub callback_id: Option<u64>,
    pub task: Task,
}
// The immediate result for that command query
pub struct CommandQueryResult {
    pub task: Option<Task>,
}

impl CommandQueryResult {
    // Create a new query result from a specific command
    pub fn new(task: Task) -> Self {
        Self { task: Some(task) }
    }
    // Explicitly tell this command query result to send the result immediately
    pub fn send(mut self) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = CommandQuery {
            task,
            thread_id: std::thread::current().id(),
            callback_id: None,
        };
        command(CommandQueryType::Single(query));
    }
    // Set callback for this specific command query result. It will receive a notif from the main thread when to execute this callback
    pub fn with_callback(mut self, callback_id: u64) {
        // Send the command
        let task = self.task.take().unwrap();
        let query = CommandQuery {
            task,
            thread_id: std::thread::current().id(),
            callback_id: Some(callback_id),
        };
        command(CommandQueryType::Single(query));
    }
}

impl std::ops::Drop for CommandQueryResult {
    // Custom drop function that actually sends the command, just in case where we did not explicitly do that
    fn drop(&mut self) {
        // Send the command
        match self.task.take() {
            Some(task) => {
                let query = CommandQuery {
                    task,
                    thread_id: std::thread::current().id(),
                    callback_id: None,
                };
                command(CommandQueryType::Single(query));
            }
            None => { /* We have called the with_callback function, so the task is empty */ }
        }
    }
}

// Initialize the main channels on the main thread
pub fn initialize_channels_main() {
    // Create the channels
    let (tx_command, rx_command) = std::sync::mpsc::channel::<CommandQueryType>();
    let mut receiver_ = RECEIVER.lock().unwrap();
    // Set the main thread values
    *receiver_ = Some(WorldTaskReceiver {
        rx: rx_command,
        lsc_txs: HashMap::new(),
    });
    // And then the worker thread template values
    let mut sender_copy_ = crate::system::SENDER_COPY.as_ref().lock().unwrap();
    let sender_copy = &mut *sender_copy_;
    *sender_copy = Some(tx_command);
    // This is indeed the main thread
    IS_MAIN_THREAD.with(|x| x.set(true));
    println!("Initialized the channels on the MainThread");
}
// Initialize the channels on a worker thread (This must be called on the worker thread)
pub fn initialize_channels_worker_thread() {
    crate::system::SENDER.with(|x| {
        let mut sender_ = x.borrow_mut();
        let sender = &mut *sender_;
        let sender_copy_ = crate::system::SENDER_COPY.as_ref().lock().unwrap();
        let sender_copy = sender_copy_.as_ref().unwrap();
        // We do the cloning
        *sender = Some(sender_copy.clone());
    })
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread(world: &mut crate::world::World, pipeline_start_data: &PipelineStartData) {
    // Poll each command query
    let receiver_ = RECEIVER.lock().unwrap();
    let receiver = receiver_.as_ref().unwrap();
    let rx = &receiver.rx;
    for query in rx.try_recv() {
        // Just execute the task
        excecute_query(query, world, receiver);
    }
    // Receive the messages from the Render Thread
    for render_thread_message in pipeline_start_data.rx.try_iter() {
        match render_thread_message {
            rendering::MainThreadMessage::ExecuteGPUObjectCallback(id, args, thread_id) => {
                // We must explicitly run the callback
                crate::system::send_lsc(
                    LogicSystemCommand::RunCallback(id, LogicSystemCallbackArguments::RenderingCommanGPUObject(args)),
                    &thread_id,
                    receiver,
                );
            }
            rendering::MainThreadMessage::ExecuteExecutionCallback(id, thread_id) => {
                // We must explicitly run the callback
                crate::system::send_lsc(
                    LogicSystemCommand::RunCallback(id, LogicSystemCallbackArguments::RenderingCommanExecution),
                    &thread_id,
                    receiver,
                );
            }
        }
    }
}
// Send a command query (or a batch command query) to the world
pub(crate) fn command(query: CommandQueryType) {
    // Check if we are running on the main thread
    let is_main_thread = IS_MAIN_THREAD.with(|x| x.get());
    if is_main_thread {
        // This is the main thread calling, we don't give a  f u c k
        let mut world = crate::world::world_mut();
        // Execute the task on the main thread
        let receiver_ = RECEIVER.lock().unwrap();
        let receiver = receiver_.as_ref().unwrap();
        excecute_query(query, &mut world, receiver)
    } else {
        // Send the command query
        SENDER.with(|sender| {
            let sender_ = sender.borrow();
            let tx = sender_.as_ref().unwrap();
            tx.send(query).unwrap();
        });
    }
}
