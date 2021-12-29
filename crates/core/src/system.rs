use crate::batch::{BatchCommandQuery, BatchManager};
use crate::command::CommandQueryResult;
use crate::communication::{WorldTaskReceiver, RECEIVER};
use crate::global::callbacks::{CallbackType, LogicSystemCallbackArguments};
use ecs::{CustomSystemData, SystemData};
use lazy_static::lazy_static;
use others::callbacks::MutCallback;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

// Some special system commands that are sent from the main thread and received on the worker threads
#[derive(Clone)]
pub enum LogicSystemCommand {
    StartSystemLoop,
    RunCallback(u64, LogicSystemCallbackArguments),
    AddEntityToSystem(usize),
    RemoveEntityFromSystem(usize),
}

lazy_static! {
    // The number of systems
    pub static ref SYSTEM_COUNTER: AtomicU8 = AtomicU8::new(0);
    // A copy of the task Sender
    pub static ref SENDER_COPY: Arc<Mutex<Option<Sender<crate::command::CommandQueryType>>>> = Arc::new(Mutex::new(None));
}

// The system group thread data is local to each system thread
thread_local! {
    pub static IS_MAIN_THREAD: Cell<bool> = Cell::new(false);
    // Sender of tasks. Is called on the worker threads, sends message to the main thread
    pub static SENDER: RefCell<Option<Sender<crate::command::CommandQueryType>>> = RefCell::new(None);
    // Each system contains a local batch manager
    static BATCH_MANAGER: RefCell<BatchManager> = RefCell::new(BatchManager::default());
}

// Create a worker thread
pub fn create_worker_thread<F, T: ecs::CustomSystemData + 'static>(default_state: T, callback: F) -> (JoinHandle<()>, usize)
where
    F: FnOnce() -> ecs::System<T> + 'static + Send,
    T: Sync + Send,
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
                let mut pre_loop_buffer: Vec<LogicSystemCommand> = Vec::new();
                // Wait for the message allowing us to start the loop
                'ack: loop {
                    match lsc_rx.try_recv() {
                        Ok(x) => match x {
                            LogicSystemCommand::StartSystemLoop => {
                                break 'ack;
                            }
                            x => {
                                pre_loop_buffer.push(x);
                            }
                        },
                        Err(x) => {}
                    };
                }
                // Create the shared data
                let mut data = default_state;
                let mut shared = SystemData::new(&mut data);
                loop {
                    // Wait for the start of the sync at the start of the frame
                    barrier_data.thread_sync();
                    // --- START OF THE SYSTEM FRAME ---
                    let i = std::time::Instant::now();
                    // Get the entities at the start of each frame
                    let ptrs = {
                        let w = crate::world::world();
                        // We must use unsafe code because if we don't we're going to execute run_system while having the world already borrowed,
                        // and RwLocks are not re-entrant so that will cause a deadlock
                        let entities = entity_ids
                            .iter()
                            .map(|x| {
                                let entity = w.ecs_manager.entitym.entity(*x).unwrap();
                                entity as *const ecs::Entity
                            })
                            .collect::<Vec<*const ecs::Entity>>();
                        entities
                    };

                    // Run the system
                    let entities = ptrs.into_iter().map(|x| unsafe { &*x }).collect::<Vec<&ecs::Entity>>();
                    system.run_system(&mut shared, &entities);

                    // --- End of the frame ---
                    // Check the start buffer first since it has priority
                    if pre_loop_buffer.len() > 0 {
                        let taken = std::mem::take(&mut pre_loop_buffer);
                        for x in taken {
                            // Execute the logic system command
                            logic_system_command(x, &mut entity_ids, &mut system, &mut shared);
                        }
                    }
                    // Check if we have any system commands that must be executed
                    match lsc_rx.try_recv() {
                        Ok(lsc) => {
                            // Execute the logic system command
                            logic_system_command(lsc, &mut entity_ids, &mut system, &mut shared);
                        }
                        Err(_) => {}
                    }

                    // Print the system frame stats
                    if system.show_stats {
                        println!("Took {}ms to execute system '{:.2}'", system.name, i.elapsed().as_secs_f32() * 1000.0);
                    }

                    // Very very end of the frame
                    if barrier_data.is_world_valid() {
                        let thread_id = std::thread::current().id();
                        // Check if the world got yeeted
                        if barrier_data.is_world_destroyed() {
                            println!("Loop for '{}' has stopped!", std::thread::current().name().unwrap());
                            barrier_data.thread_sync_quit();
                            break;
                        }
                        others::barrier::as_ref().thread_sync();
                        // Wait until the main world gives us permission to continue
                        others::barrier::as_ref().thread_sync_local_callbacks(&thread_id);
                        // We got permission, we can run the local callbacks
                        crate::callbacks::execute_local_callbacks();
                        // Tell the main thread we have finished executing thread local callbacks
                        others::barrier::as_ref().thread_sync_local_callbacks(&thread_id);
                    }
                }
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

// Execute a logic system command
fn logic_system_command<T: CustomSystemData>(lsc: LogicSystemCommand, entity_ids: &mut Vec<usize>, system: &mut ecs::System<T>, shared: &mut SystemData<T>) {
    match lsc {
        LogicSystemCommand::RunCallback(id, result_data) => {
            // We will run this when we run the local callbacks!
            crate::callbacks::buffer_callback_execution(id, result_data);
        }
        LogicSystemCommand::AddEntityToSystem(entity_id) => {
            // Add the entity to the current entity list
            let ptr = {
                let w = crate::world::world();
                let entity = w.ecs_manager.entitym.entity(entity_id).unwrap();
                entity as *const ecs::Entity
            };
            entity_ids.push(entity_id);
            let entity = unsafe { ptr.as_ref().unwrap() };
            system.add_entity(shared, entity);
        }
        LogicSystemCommand::RemoveEntityFromSystem(entity_id) => {
            // Remove the entity from the current entity list
            let ptr = {
                let w = crate::world::world();
                entity_ids.retain(|x| *x != entity_id); // We know that there is a unique entity ID in here, so no need to worry about duplicates
                let entity = w.ecs_manager.entitym.entity(entity_id).unwrap();
                entity as *const ecs::Entity
            };
            let entity = unsafe { ptr.as_ref().unwrap() };
            system.remove_entity(shared, entity);
            // The main thread does not know that we have deleted the entity from this entity, so we must decrement the counter
            crate::command::CommandQueryResult::new(crate::tasks::Task::EntityRemovedDecrementCounter(entity_id)).send();
        }
        LogicSystemCommand::StartSystemLoop => { /* How the fuck */ }
    }
}

// Send a LogicSystemCommand to a specific thread
pub fn send_lsc(lgc: LogicSystemCommand, thread_id: &std::thread::ThreadId, receiver: &WorldTaskReceiver) {
    // Get the sender
    let senders = &receiver.lsc_txs;
    let sender = senders.get(thread_id).unwrap();
    // Send the message
    sender.send(lgc).unwrap();
}
// Send a LogicSystemCommand to all the threads
pub fn send_lsc_all(lgc: LogicSystemCommand, receiver: &WorldTaskReceiver) {
    // Get the senders
    let senders = &receiver.lsc_txs;
    for (h, sender) in senders {
        // Send the message
        sender.send(lgc.clone()).unwrap();
    }
}

// Add a command onto a batch
pub fn batch_add(batch_id: u32, command_result: CommandQueryResult) {
    BATCH_MANAGER.with(|cell| {
        let mut cell = cell.borrow_mut();
        let batch = cell.batches.entry(batch_id).or_default();
        batch.add(command_result);
    });
}

// Send a batch to the main thread for execution
pub fn send_batch(batch_id: u32, delete: bool) {
    BATCH_MANAGER.with(|cell| {
        let mut cell = cell.borrow_mut();
        if delete {
            // We must delete the batch after sending
            if let Option::Some(mut batch) = cell.batches.remove(&batch_id) {
                batch.send();
            }
        } else {
            // We should no delete the batch
            if let Option::Some(batch) = cell.batches.get_mut(&batch_id) {
                batch.send();
            }
        }
    });
}
