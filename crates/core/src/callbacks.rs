use lazy_static::lazy_static;
use others::callbacks::*;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{atomic::AtomicU64, Mutex}, thread::LocalKey,
};
use crate::world::World;

lazy_static! {
    static ref CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0); // The number of callbacks that have been created
}

// Get a specific callback on this thread
pub fn get_callback(id: u64, callback_manager: &mut CallbackManagerBuffer) -> CallbackType {
    callback_manager.callbacks.remove(&id).unwrap()
}
// Get a specific local callback on this thread
pub fn get_local_callback(id: u64, callback_manager: &mut CallbackManagerBuffer) -> (CallbackType, LocalCallbackArguments) {
    let callback = callback_manager.callbacks.remove(&id).unwrap();
    let args = callback_manager.local_callback_arguments.remove(&id).unwrap();
    (callback, args)
}
// The main callback manager that is stored on the main thread, and that sends commands to the system threads that must execute their callbacks
// Callback manager that contains all the current callbacks (Thread Local)
pub struct CallbackManagerBuffer
{
    callbacks: HashMap<u64, CallbackType>,
    local_callback_arguments: HashMap<u64, LocalCallbackArguments>,
}

impl Default for CallbackManagerBuffer
{
    fn default() -> Self {
        Self { callbacks: HashMap::new(), local_callback_arguments: HashMap::new() }
    }
}

impl CallbackManagerBuffer
{
    // Add a callback to this thread local buffer
    pub fn add_callback(&mut self, id: u64, callback: CallbackType) {
        self.callbacks.insert(id, callback);
    }
    // Add a local callback to this thread local buffer
    pub fn add_local_callback(&mut self, id: u64, callback: CallbackType, local_callback_arguments: LocalCallbackArguments) {
        self.add_callback(id, callback);
        self.local_callback_arguments.insert(id, local_callback_arguments);
    }
}

// Per thread
thread_local! {
    static CALLBACK_MANAGER_BUFFER: Mutex<CallbackManagerBuffer> = Mutex::new(CallbackManagerBuffer::default());
}

// Execute a specific callback on this thread
pub fn execute_callback(id: u64, arguments: LogicSystemCallbackArguments, world: &mut crate::world::World) {
    // Get the callback arguments from teh result data
    CALLBACK_MANAGER_BUFFER.with(|mutex| {
        let mut callback_manager_ = mutex.lock().unwrap();
        let callback_manager = &mut *callback_manager_;

        // Get the callback
        let callback = get_callback(id, callback_manager);
        match callback {
            CallbackType::GPUObjectCallback(x) => {
                let callback = x.callback.as_ref();
                // Make sure this callback is the GPUObject one
                if let LogicSystemCallbackArguments::RenderingGPUObject(gpuobject) = arguments {
                    (callback)(gpuobject);
                }
            }
            CallbackType::EntityCreatedCallback(x) => {
                let callback = x.callback.as_ref();
                // Make sure this callback is the EntityRef one
                if let LogicSystemCallbackArguments::EntityRef(entity_id) = arguments {
                    let entity = world.ecs_manager.entitym.entity(entity_id);
                    (callback)(entity);
                }
            }
            /* #region Local callbacks */
            CallbackType::LocalEntityMut(_) => {}
            /* #endregion */
        }
    });
}
// Execute a local callback
pub fn execute_local_callback(id: u64) {
    CALLBACK_MANAGER_BUFFER.with(|mutex| {
        let mut callback_manager_ = mutex.lock().unwrap();
        let callback_manager = &mut *callback_manager_;

        // Get the world mut callback
        let (callback, arguments) = get_local_callback(id, callback_manager);
        match callback {
            CallbackType::LocalEntityMut(entity_callback) => {
                // Get the local callback arguments that are necessary
                if let LocalCallbackArguments::EntityMut(entity_id) = arguments {
                    // Get the mut entity
                    let callback = entity_callback.callback.as_ref();                            
                    let mut cloned_entity = {
                        let w = crate::world::world();
                        w.ecs_manager.entitym.entity(entity_id).clone()
                    };
                    (callback)(&mut cloned_entity);
                    // Update the value in the world
                    let mut w = crate::world::world_mut();
                    let entity = w.ecs_manager.entitym.entity_mut(entity_id);
                    *entity = cloned_entity;
                } 
            },   
            _ => {}
        }
    });
}

// The data that will be sent back to the logic system from the main thread
#[derive(Clone)]
pub enum LogicSystemCallbackArguments {
    // Entity
    EntityRef(usize),
    // Rendering
    RenderingGPUObject(rendering::GPUObject),
}

// Arguments used when calling the local callbacks
pub enum LocalCallbackArguments {
    EntityMut(usize),
}

// The callback type
pub enum CallbackType {
    GPUObjectCallback(OwnedCallback<rendering::GPUObject>),
    EntityCreatedCallback(RefCallback<ecs::Entity>),
    LocalEntityMut(MutCallback<ecs::Entity>),
}

impl CallbackType {
    // Turn this callback into a callback ID after adding it to thread local callback manager buffer
    pub fn create(self) -> u64 {
        let id = CALLBACK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        CALLBACK_MANAGER_BUFFER.with(|x| {
            let mut manager_ = x.lock().unwrap();
            let manager = &mut *manager_;
            manager.add_callback(id, self);
        });
        id
    }
}
