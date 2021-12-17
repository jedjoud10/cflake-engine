use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicU64, borrow::BorrowMut};
use lazy_static::lazy_static;

lazy_static! {
    static ref CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0); // The number of callbacks that have been created
}

// Per thread
thread_local! {
    static CALLBACK_MANAGER_BUFFER: RefCell<CallbackManagerBuffer> = RefCell::new(CallbackManagerBuffer::default());
}

// Execute a specific callback on this thread
pub fn execute_callback(id: u64, result_data: LogicSystemCallbackResultData) {
    let mut world = crate::world::world_mut(); 
    CALLBACK_MANAGER_BUFFER.with(|cell| {
        let mut callback_manager = cell.borrow_mut();
        match callback_manager.callbacks.remove(&id) {
            Some(callback_type) => {
                // Run the callback type
                match callback_type {
                    CallbackType::None => { /* No callbacks */ },
                    CallbackType::EntityRefCallbacks(x) => {
                        let callback = x.callback.as_ref();
                        // Make sure this callback is the EntityRef one
                        if let LogicSystemCallbackResultData::EntityRef(entity_id) = result_data {
                            let entity = world.ecs_manager.entitym.entity(entity_id);
                            (callback)(entity);
                        }
                    },
                    CallbackType::EntityMutCallbacks(_) => todo!(),
                    CallbackType::ComponentMutCallbacks(_) => todo!(),
                }
            },
            None => { /* H o w */ },
        }
    });
}

// The data that will be sent back to the logic system from the main thread
pub enum LogicSystemCallbackResultData {
    // Entity
    EntityRef(usize),
    EntityMut(usize),
}

// The main callback manager that is stored on the main thread, and that sends commands to the system threads that must execute their callbacks 
// Callback manager that contains all the current callbacks (Thread Local)
#[derive(Default)]
pub struct CallbackManagerBuffer {
    callbacks: HashMap<u64, CallbackType>
}

impl CallbackManagerBuffer {
    // Add a callback to this thread local buffer
    pub fn add_callback(&mut self, id: u64, callback: CallbackType) {
        self.callbacks.insert(id, callback);
    }
}

// The callback type
pub enum CallbackType {
    None,
    EntityRefCallbacks(RefCallback<ecs::Entity>),
    EntityMutCallbacks(MutCallback<ecs::Entity>),
    ComponentMutCallbacks(MutCallback<Box<dyn ecs::ComponentInternal + Send + Sync>>),
}

impl CallbackType {
    // Create the callback and get back it's ID
    pub fn create(self) -> u64 {
        let id = CALLBACK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        CALLBACK_MANAGER_BUFFER.with(|cell| {
            let mut callback_manager = cell.borrow_mut();
            callback_manager.add_callback(id, self);
        });
        id
    }
}

// The callback sending data that will actually be sent to the main thread using the command
pub enum CallbackSendingData {
    None,
    ValidCallback(u64)
}

// A ref callback, always ran at the end of the current system frame
pub struct RefCallback<T> {
    pub callback: Box<dyn Fn(&T)>,
}
// A mutable callback that mutates that value passed. Always ran at the end of the world thread frame
pub struct MutCallback<T> {
    pub callback: Box<dyn Fn(&mut T)>,
}

impl<T> RefCallback<T> {
    pub fn new<F>(c: F) -> Self where F: Fn(&T) + 'static {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> MutCallback<T> {
    pub fn new<F>(c: F) -> Self where F: Fn(&mut T) + 'static {
        let callback = Box::new(c);
        Self { callback }
    }
}