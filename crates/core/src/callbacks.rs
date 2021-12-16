use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicU64};
use lazy_static::lazy_static;

lazy_static! {
    static ref CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0); // The number of callbacks that have been created
}

// Per thread
thread_local! {
    static CALLBACK_MANAGER_BUFFER: RefCell<CallbackManagerBuffer> = RefCell::new(CallbackManagerBuffer::default());
}


// Callback manager that contains all the current callbacks
#[derive(Default)]
pub struct CallbackManagerBuffer {
    pub callbacks: HashMap<u64, CallbackType>
}

// The callback type
pub enum CallbackType {
    None,
    EntityRefCallbacks(RefCallback<ecs::Entity>),
    EntityMutCallbacks(MutCallback<ecs::Entity>),
    ComponentMutCallbacks(MutCallback<Box<dyn ecs::ComponentInternal + Send + Sync>>),
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
    pub fn new(boxed_callback: Box<dyn Fn(&T)>) -> Self {
        Self { callback: boxed_callback }
    }
}

impl<T> MutCallback<T> {
    pub fn new(boxed_callback: Box<dyn Fn(&mut T)>) -> Self {
        Self { callback: boxed_callback }
    }
}