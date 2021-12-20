use crate::communication::{WorldTaskReceiver, RECEIVER};
use crate::global::callbacks::LogicSystemCallbackArguments;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex, mpsc::{Sender}};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;

// Some special system commands that are sent from the main thread and received on the worker threads
pub enum LogicSystemCommand {
    RunCallback(u64, LogicSystemCallbackArguments),
    AddEntityToSystem(usize),
    RemoveEntityFromSystem(usize),
}

lazy_static! {
    // The number of systems
    pub static ref SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
    // A copy of the task Sender
    pub static ref SENDER_COPY: Arc<Mutex<Option<Sender<crate::command::CommandQuery>>>> = Arc::new(Mutex::new(None));
}

// The system group thread data is local to each system thread
thread_local! {
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub static SENDER: RefCell<Option<Sender<crate::command::CommandQuery>>> = RefCell::new(None);
    static WORLDMUT_CALLBACK_IDS: RefCell<Vec<u64>> = RefCell::new(Vec::new());
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData>(callback: F) -> (JoinHandle<()>, usize)
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
{
    let system_id = SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let builder = std::thread::Builder::new().name(format!("LogicSystemThread '{}'", system_id));
    let (tx_cbitfield, rx_cbitfield) = std::sync::mpsc::channel::<usize>();
    // Simple one way channel (MainThread -> Worker Thread)
    let (tx, rx) = std::sync::mpsc::channel::<LogicSystemCommand>();
    let handler = builder
        .spawn(move || {
            // We must initialize the channels
            crate::command::initialize_channels_worker_thread();
            // Set our render command sender as well
            rendering::pipec::initialize_threadlocal_render_comms();
            // Create the system on this thread
            SENDER.with(|x| {
                let mut system = callback();
                // Send the data about this system to the main thread
                let sender_ = x.borrow();
                let sender = sender_.as_ref().unwrap();
                let lsc_rx = rx;
                println!("Hello from '{}'!", std::thread::current().name().unwrap());
                let barrier_data = others::barrier::as_ref();
                // Start the system loop
                let mut entity_ids: Vec<usize> = Vec::new();
                tx_cbitfield.send(system.c_bitfield).unwrap();
                loop {
                    {
                        let i = std::time::Instant::now();
                        // Get the entities at the start of each frame
                        let ptrs = {
                            let w = crate::world::world();
                            let entities = entity_ids
                                .iter()
                                .map(|x| {
                                    let entity = w.ecs_manager.entitym.entity(*x);

                                    entity as *const ecs::Entity
                                })
                                .collect::<Vec<*const ecs::Entity>>();
                            entities
                        };

                        // Start of the independent system frame
                        // End of the independent system frame, we must wait until the main thread allows us to continue
                        // Check if the system is still running
                        // --- Start of the frame ---
                        let entities = ptrs.iter().map(|x| unsafe { x.as_ref().unwrap() }).collect::<Vec<&ecs::Entity>>();
                        system.run_system(&entities);
                        

                        // --- End of the frame ---
                        // Check if we have any system commands that must be executed
                        match lsc_rx.try_recv() {
                            Ok(lsc) => {
                                // Execute the logic system command
                                match lsc {
                                    LogicSystemCommand::RunCallback(id, result_data) => {
                                        let mut w = crate::world::world_mut();
                                        let world = &mut *w;
                                        crate::callbacks::execute_callback(id, result_data, world);
                                    }
                                    LogicSystemCommand::AddEntityToSystem(entity_id) => {
                                        // Add the entity to the current entity list
                                        let ptr = {
                                            let w = crate::world::world();
                                            let entity = w.ecs_manager.entitym.entity(entity_id);
                                            entity as *const ecs::Entity
                                        };
                                        entity_ids.push(entity_id);
                                        let entity = unsafe { ptr.as_ref().unwrap() };
                                        system.add_entity(entity);
                                    }
                                    LogicSystemCommand::RemoveEntityFromSystem(entity_id) => {
                                        // Remove the entity from the current entity list
                                        let ptr = {
                                            let w = crate::world::world();
                                            entity_ids.retain(|x| *x != entity_id); // We know that there is a unique entity ID in here, so no need to worry about duplicates
                                            let entity = w.ecs_manager.entitym.entity(entity_id);
                                            entity as *const ecs::Entity
                                        };
                                        let entity = unsafe { ptr.as_ref().unwrap() };
                                        system.remove_entity(entity);
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                        // Now we can run the MutCallback<World> that we have created during the frame
                        WORLDMUT_CALLBACK_IDS.with(|cell| {
                            let mut callbacks_ = cell.borrow_mut();
                            let callbacks = &mut *callbacks_;
                            // No need to waste trying to get world mut in this case
                            if callbacks.len() > 0 {
                                let cleared_callbacks = std::mem::take(callbacks);
                                let mut w = crate::world::world_mut();
                                let world = &mut *w;
                                for id in cleared_callbacks {
                                    crate::callbacks::execute_world_mut_callback(id, world);
                                }
                            }
                        });
                    }

                    // Very very end of the frame
                    if barrier_data.is_world_valid() {
                        // First sync
                        barrier_data.thread_sync();
                        if barrier_data.is_world_destroyed() {
                            barrier_data.thread_sync_quit();
                            break;
                        }
                        // Second sync
                        barrier_data.thread_sync();
                    }
                }
                println!("Loop for '{}' has stopped!", std::thread::current().name().unwrap());
            });
        })
        .unwrap();
    // Wait for the worker thread to send us the data back
    let c_bitfield = rx_cbitfield.recv().unwrap();
    // Add the tx
    let mut receiver_ = RECEIVER.lock().unwrap();
    let receiver = receiver_.as_mut().unwrap();
    receiver.lsc_txs.insert(handler.thread().id(), tx);
    (handler, c_bitfield)
}

// Add a world mut callback ID to the thread local vector
pub fn add_worldmutcallback_id(id: u64) {
    WORLDMUT_CALLBACK_IDS.with(|cell| {
        let mut callbacks_ = cell.borrow_mut();
        let callbacks = &mut *callbacks_;
        callbacks.push(id);
    })
}

// Send a LogicSystemCommand to a specific thread
pub fn send_lsc(lgc: LogicSystemCommand, thread_id: &std::thread::ThreadId, receiver: &WorldTaskReceiver) {
    // Get the sender
    let senders = &receiver.lsc_txs;
    let sender = senders.get(thread_id).unwrap();
    // Send the message
    sender.send(lgc).unwrap();
}
