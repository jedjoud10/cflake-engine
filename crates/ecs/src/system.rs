use std::{any::Any, sync::{Arc, atomic::{AtomicPtr, Ordering}}, rc::Rc};

use crate::{ComponentID, Entity, impl_systemdata};

#[derive(Default)]
// Manages the systems, however each system is in it's own thread (For now at least)
pub struct SystemManager {
    pub systems: Vec<SystemThreadData>,
}

// Contains some data about the actual system on the worker thread
pub struct SystemThreadData {
    pub join_handle: std::thread::JoinHandle<()>,
    pub c_bitfield: usize,
}

impl SystemThreadData {
    // New
    pub fn new(join_handle: std::thread::JoinHandle<()>, c_bitfield: usize) -> Self {
        Self { join_handle, c_bitfield }
    }
}

// A system event enum
pub enum SystemEventType<T> where T: CustomSystemData {
    // Control events
    SystemPrefire(fn(&mut SystemData<T>)),
    SystemPostfire(fn(&mut SystemData<T>)),
    // Entity events
    EntityAdded(fn(&mut SystemData<T>, &Entity)),
    EntityRemoved(fn(&mut SystemData<T>, &Entity)),
    EntityUpdate(fn(&mut SystemData<T>, &Entity)),
}

// A system, stored on the stack, but it's SystemData is a trait object
pub struct System<T>
where
    T: CustomSystemData,
{
    pub starting_custom_data_state: Option<T>, // The starting state of the custom system data. This will not be given to the system events
    pub c_bitfield: usize,

    // Events
    // Control events
    system_prefire: Option<fn(&mut SystemData<T>)>,
    system_postfire: Option<fn(&mut SystemData<T>)>,
    // Entity events
    entity_added: Option<fn(&mut SystemData<T>, &Entity)>,
    entity_removed: Option<fn(&mut SystemData<T>, &Entity)>,
    entity_update: Option<fn(&mut SystemData<T>, &Entity)>,
}

// Initialization of the system
impl<T> System<T>
where
    T: CustomSystemData,
{
    // Create a new system
    pub fn new(custom_data: T) -> Self {
        System {
            starting_custom_data_state: Some(custom_data),
            c_bitfield: 0,
            system_prefire: None,
            system_postfire: None,
            entity_added: None,
            entity_removed: None,
            entity_update: None,
        }
    }
}

// System code
impl<T> System<T>
where
    T: CustomSystemData,
{
    // Add a component to this system's component bitfield id
    pub fn link<U: ComponentID>(&mut self) {
        let c = crate::registry::get_component_id::<U>();
        self.c_bitfield |= c;
    }
    // Attach the a specific system event
    pub fn event(&mut self, event: SystemEventType<T>) {
        match event {
            // Control events
            SystemEventType::SystemPrefire(x) => self.system_prefire = Some(x),
            SystemEventType::SystemPostfire(x) => self.system_postfire = Some(x),
            // Entity events
            SystemEventType::EntityAdded(x) => self.entity_added = Some(x),
            SystemEventType::EntityRemoved(x) => self.entity_removed = Some(x),
            SystemEventType::EntityUpdate(x) => self.entity_update = Some(x),
        };
    }
    // Add an entity to the current system
    pub fn add_entity(&mut self, shared: &mut SystemData<T>, entity: &Entity) {
        // Fire the event
        match self.entity_added {
            Some(entity_added_evn) => entity_added_evn(shared, entity),
            None => {}
        }
    }
    // Remove an entity from the current system
    pub fn remove_entity(&mut self, shared: &mut SystemData<T>, entity: &Entity) {
        // Fire the event
        match self.entity_removed {
            Some(entity_removed_evn) => entity_removed_evn(shared, entity),
            None => {}
        }
    }
    // Stop the system permanently
    pub fn end_system(&mut self, shared: &mut SystemData<T>, entities: &Vec<&Entity>) {
        match self.entity_removed {
            Some(entity_removed) => {
                // Fire the entity removed event
                for entity in entities.iter() {
                    entity_removed(shared, entity);
                }
            }
            None => {}
        }
    }
    // Run the system for a single iteration
    pub fn run_system(&mut self, shared: &mut SystemData<T>, entities: &Vec<&Entity>) {
        // Pre fire event
        match self.system_prefire {
            Some(system_prefire_event) => system_prefire_event(shared),
            None => {}
        }

        // Only update the entities if we have a a valid event. No need to waste time updating them ¯\_(ツ)_/¯
        match self.entity_update {
            Some(entity_update) => {
                // Loop over all the entities and fire the event
                for entity in entities.iter() {
                    entity_update(shared, entity);
                }
            }
            None => {}
        }

        // Post fire event
        match self.system_postfire {
            Some(system_postfire) => system_postfire(shared),
            None => {}
        }
    }
}

// Trait for custom system data
pub trait CustomSystemData {}

// Some custom system data that can be copied whenever we create a callback
pub struct SystemData<T> where T: CustomSystemData {
    pub ptr: Rc<*mut T>
}

impl<T> Clone for SystemData<T> where T: CustomSystemData {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone() }
    }
}

impl<T> SystemData<T> where T: CustomSystemData {
    // New
    pub fn new(t: &mut T) -> Self {
        Self {
            ptr: Rc::new(t as *mut T)
        }
    }
}

impl<T> std::ops::Deref for SystemData<T> where T: CustomSystemData {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &**self.ptr }
    }
}

impl<T> std::ops::DerefMut for SystemData<T> where T: CustomSystemData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut().unwrap() }
    }
}