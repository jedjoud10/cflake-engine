use crate::communication::{WorldTaskReceiver, WorldTaskSender, RECEIVER};
use crate::global::callbacks::LogicSystemCallbackArguments;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
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
}

// The system group thread data is local to each system thread
thread_local! {
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub static SENDER: RefCell<Option<WorldTaskSender>> = RefCell::new(None);
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData>(callback: F) -> (JoinHandle<()>, usize)
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
{
    let system_id = SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let builder = std::thread::Builder::new().name(format!("LogicSystemThread '{}'", system_id));
    let (tx, rx) = std::sync::mpsc::channel::<usize>();
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
                tx.send(system.c_bitfield).unwrap();
                let sender_ = x.borrow();
                let sender = sender_.as_ref().unwrap();
                let lsc_rx = &sender.lsc_rx;
                println!("Hello from '{}'!", std::thread::current().name().unwrap());
                let barrier_data = others::barrier::as_ref();
                // Start the system loop
                let mut entity_ids: Vec<usize> = Vec::new();
                loop {
                    {
                        println!("Run system!");
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
    let c_bitfield = rx.recv().unwrap();
    // Add the tx
    let mut receiver_ = RECEIVER.lock().unwrap();
    let receiver = receiver_.as_mut().unwrap();
    receiver.lsc_txs.insert(handler.thread().id(), receiver.template_wtc_tx.clone());
    (handler, c_bitfield)
}

// Send a LogicSystemCommand to a specific thread
pub fn send_lsc(lgc: LogicSystemCommand, thread_id: &std::thread::ThreadId, receiver: &WorldTaskReceiver) {
    // Get the sender
    let senders = &receiver.lsc_txs;
    let sender = senders.get(thread_id).unwrap();
    // Send the message
    sender.send(lgc).unwrap();
}
