use ecs::stored::{Stored, StoredMut};
use lazy_static::lazy_static;
use others::callbacks::*;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{atomic::AtomicU64, Mutex},
};

use crate::world::World;

// Per thread
thread_local! {
    static CALLBACK_MANAGER_BUFFER: Mutex<CallbackManagerBuffer<CallbackType>> = Mutex::new(CallbackManagerBuffer::default());
}

// Execute a specific callback on this thread
pub fn execute_callback(id: u64, arguments: LogicSystemCallbackArguments, world: &mut crate::world::World) {
    // Get the callback arguments from teh result data
    CALLBACK_MANAGER_BUFFER.with(|mutex| {
        let mut callback_manager_ = mutex.lock().unwrap();
        let callback_manager = &mut *callback_manager_;

        // Get the callback
        let callback = get_callback::<CallbackType>(id, callback_manager);
        match callback {
            CallbackType::GPUObjectCallback(x) => {
                let callback = x.callback.as_ref();
                // Make sure this callback is the GPUObject one
                if let LogicSystemCallbackArguments::RenderingGPUObject(gpuobject) = arguments {
                    (callback)(gpuobject);
                }
            }
            CallbackType::EntityRefCallbacks(x) => {
                let callback = x.callback.as_ref();
                // Make sure this callback is the EntityRef one
                if let LogicSystemCallbackArguments::EntityRef(entity_id) = arguments {
                    let entity = world.ecs_manager.entitym.entity(entity_id);
                    (callback)(entity);
                }
            }
            CallbackType::WorldMut(x) => { /* This is not where we execute MutCallback<World> */ }
        }
    });
}
// Execute the world mut callback
pub fn execute_world_mut_callback(id: u64, world: &mut crate::world::World) {
    CALLBACK_MANAGER_BUFFER.with(|mutex| {
        let mut callback_manager_ = mutex.lock().unwrap();
        let callback_manager = &mut *callback_manager_;

        // Get the world mut callback
        let callback = get_callback::<CallbackType>(id, callback_manager);
        match callback {
            CallbackType::WorldMut(x) => {
                let callback = x.callback.as_ref();
                (callback)(world);
            }
            _ => {}
        }
    });
}

// The data that will be sent back to the logic system from the main thread
#[derive(Clone)]
pub enum LogicSystemCallbackArguments {
    // Entity
    EntityRef(usize),
    EntityMut(usize),
    // Rendering
    RenderingGPUObject(rendering::GPUObject),
}

// The callback type
pub enum CallbackType {
    GPUObjectCallback(OwnedCallback<rendering::GPUObject>),
    EntityRefCallbacks(RefCallback<ecs::Entity>),
    WorldMut(MutCallback<World>),
}

impl others::callbacks::Callback for CallbackType {
    fn create(self) -> u64 {
        others::callbacks::create_callback_internal(self, &CALLBACK_MANAGER_BUFFER)
    }
}
