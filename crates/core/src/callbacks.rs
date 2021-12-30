use ecs::{SystemData, identifiers::EntityID};
use lazy_static::lazy_static;
use others::callbacks::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
        Arc, Mutex,
    },
};

lazy_static! {
    static ref CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0); // The number of callbacks that have been created
}

thread_local! {
    static CALLBACK_MANAGER_BUFFER: RefCell<CallbackManagerBuffer> = RefCell::new(CallbackManagerBuffer::default());
}
// The main callback manager that is stored on the main thread, and that sends commands to the system threads that must execute their callbacks
// Callback manager that contains all the current callbacks (Thread Local)
pub struct CallbackManagerBuffer {
    callbacks: HashMap<u64, CallbackType>,
    buffered_executions: Vec<(u64, LogicSystemCallbackArguments)>,
}

impl Default for CallbackManagerBuffer {
    fn default() -> Self {
        Self {
            callbacks: HashMap::new(),
            buffered_executions: Vec::new(),
        }
    }
}

impl CallbackManagerBuffer {
    // Add a callback to this thread local buffer
    pub fn add_callback(&mut self, id: u64, callback: CallbackType) {
        self.callbacks.insert(id, callback);
    }
}

// Buffer the execution of a certain callback
pub fn buffer_callback_execution(id: u64, arguments: LogicSystemCallbackArguments) {
    CALLBACK_MANAGER_BUFFER.with(|cell| {
        let mut cell = cell.borrow_mut();
        // Buffer the callback ID and the callback arguments
        cell.buffered_executions.push((id, arguments));
    })
}

// Execute all the local callbacks
pub fn execute_local_callbacks() {
    // Aaoaoao
    let mut callbacks: Vec<(CallbackType, LogicSystemCallbackArguments)> = Vec::new();
    CALLBACK_MANAGER_BUFFER.with(|cell| {
        let mut cell = cell.borrow_mut();
        // We must now empty the buffered executions
        if cell.buffered_executions.len() > 0 {
            let cleared_callbacks = std::mem::take(&mut cell.buffered_executions);
            for (id, arguments) in cleared_callbacks {
                let callback = cell.callbacks.remove(&id);
                // We might've gotten the signal to run a callback that we do not have
                if let Option::Some(callback) = callback {
                    callbacks.push((callback, arguments));
                }
            }
        }
    });

    // Now we must execute the callbacks
    for (callback, arguments) in callbacks {
        // Execute the local callbacks
        match callback {
            CallbackType::RenderingGPUObjectCallback(x) => {
                let callback = x.callback;
                // Make sure this callback is the GPUObject one
                if let LogicSystemCallbackArguments::RenderingCommanGPUObject(args) = arguments {
                    (callback)(args);
                }
            }
            CallbackType::RenderingCommandExecution(x) => {
                let callback = x.callback;
                (callback)();
            }
            CallbackType::EntityCreatedCallback(x) => {
                let callback = x.callback;
                // Make sure this callback is the EntityRef one
                if let LogicSystemCallbackArguments::EntityRef(id) = arguments {
                    let cloned_entity = {
                        let w = crate::world::world();
                        w.ecs_manager.entity(id).unwrap().clone()
                    };
                    (callback)(&cloned_entity);
                }
            }
            CallbackType::LocalEntityMut(entity_callback) => {
                // Get the local callback arguments that are necessary
                if let LogicSystemCallbackArguments::EntityMut(entity_id) = arguments {
                    // Get the mut entity
                    let callback = entity_callback.callback;
                    let mut cloned_entity = {
                        let w = crate::world::world();
                        w.ecs_manager.entity(entity_id).unwrap().clone()
                    };
                    // We must NOT have world() or world_mut() locked when executing these types of callbacks
                    (callback)(&mut cloned_entity);
                    // Update the value in the world
                    let mut w = crate::world::world_mut();
                    let entity = w.ecs_manager.entity_mut(entity_id).unwrap();
                    *entity = cloned_entity;
                }
            }
        }
    }
}

// The data that will be sent back to the logic system from the main thread
#[derive(Clone)]
pub enum LogicSystemCallbackArguments {
    // Entity
    EntityRef(EntityID),
    EntityMut(EntityID),
    // Rendering
    RenderingCommanGPUObject((rendering::GPUObject, rendering::GPUObjectID)),
    RenderingCommanExecution,
}

// The callback type
pub enum CallbackType {
    RenderingGPUObjectCallback(OwnedCallback<(rendering::GPUObject, rendering::GPUObjectID)>),
    RenderingCommandExecution(NullCallback),
    EntityCreatedCallback(RefCallback<ecs::Entity>),
    LocalEntityMut(MutCallback<ecs::Entity>),
}

impl CallbackType {
    // Turn this callback into a callback ID after adding it to thread local callback manager buffer
    pub fn create(self) -> u64 {
        let id = CALLBACK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        CALLBACK_MANAGER_BUFFER.with(|cell| {
            let mut manager_ = cell.borrow_mut();
            let manager = &mut *manager_;
            manager.add_callback(id, self);
        });
        id
    }
}
