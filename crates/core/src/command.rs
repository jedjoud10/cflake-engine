// Sending - Receiving
use crate::communication::*;
use crate::system::*;
use crate::tasks::*;
use std::{collections::HashMap, sync::atomic::Ordering};

// A sent command query
pub struct CommandQuery {
    pub thread_id: std::thread::ThreadId,
    pub task: Task,
}
impl CommandQuery {
    // From single
    pub fn new(task: Task) -> Self {
        let thread_id = std::thread::current().id();
        Self { thread_id, task }
    }
}

// The functions
pub fn initialize_channels() {
    // Create the channels
    let (tx, rx) = std::sync::mpsc::channel::<(u64, CommandQuery)>();
    let mut receiver = RECEIVER.lock().unwrap();
    let mut sender = SENDER.lock().unwrap();
    // The task senders
    receiver.txs = Some(HashMap::new());
    sender.tx = Some(tx);
    // The taskreturn senders
    receiver.rx = Some(rx);
}
// Frame tick on the main thread. Polls the current tasks and excecutes them. This is called at the end of each logic frame (16ms per frame)
pub fn frame_main_thread() {
    // Poll each command query
    let receiver = RECEIVER.lock().unwrap();
    let rx = receiver.rx.as_ref().unwrap();
    let txs = receiver.txs.as_ref().unwrap();
    let mut world = crate::world::world_mut();
    for (id, query) in rx.try_recv() {
        let waitabletask = WaitableTask {
            id,
            thread_id: query.thread_id,
            val: Some(excecute_task(query.task, &mut world)),
        };
        // Send the result to the corresponding system threads
        match txs.get(&query.thread_id) {
            Some(x) => {
                // Send the return value to the corresponding receiver
                x.send(waitabletask).unwrap();
            }
            None => { /* Not the correct thread id */ }
        }
    }
}
// Send a command query to the world, giving back a command return that can be waited for
pub fn command(query: CommandQuery) -> WaitableTask {
    // Send the command query
    let x = SENDER.lock().unwrap();
    let tx = x.tx.as_ref().unwrap();
    let id = COUNTER.fetch_add(0, Ordering::Relaxed);
    let is_main_thread = IS_MAIN_THREAD.with(|x| x.get());
    // Early main thread exit lol
    if !is_main_thread {
        tx.send((id, query)).unwrap();
        // Increment the counter
        // Get the corresponding return command value
        WaitableTask {
            id,
            thread_id: std::thread::current().id(),
            val: None,
        }
    } else {
        // This is the main thread calling, we don't give a  f u c k
        WaitableTask {
            id,
            thread_id: std::thread::current().id(),
            val: {
                let mut world = crate::world::world_mut();
                let output = excecute_task(query.task, &mut world);
                Some(output)
            },
        }
    }
}
