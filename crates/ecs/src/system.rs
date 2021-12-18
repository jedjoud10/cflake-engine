use crate::{ComponentID, Entity};

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
pub enum SystemEventType<T> {
    // Control events
    SystemPrefire(fn(&mut T)),
    SystemPostfire(fn(&mut T)),
    // Entity events
    EntityAdded(fn(&mut T, &Entity)),
    EntityRemoved(fn(&mut T, &Entity)),
    EntityUpdate(fn(&mut T, &Entity)),
}

// A system, stored on the stack, but it's SystemData is a trait object
pub struct System<T>
where
    T: CustomSystemData,
{
    custom_data: T,
    pub c_bitfield: usize,

    // Events
    // Control events
    system_prefire: Option<fn(&mut T)>,
    system_postfire: Option<fn(&mut T)>,
    // Entity events
    entity_added: Option<fn(&mut T, &Entity)>,
    entity_removed: Option<fn(&mut T, &Entity)>,
    entity_update: Option<fn(&mut T, &Entity)>,
}

// Initialization of the system
impl<T> System<T>
where
    T: CustomSystemData,
{
    // Create a new system
    pub fn new(custom_data: T) -> Self {
        System {
            custom_data,
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
        
        if crate::registry::is_component_registered::<U>() {
            // Link the component if it was already registered
            let c = crate::registry::get_component_id::<U>().unwrap();
            self.c_bitfield |= c;
        } else {
            // Link the component if it was not registered yet
            self.c_bitfield |= crate::registry::register_component::<U>();
        }
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
    pub fn add_entity(&mut self, entity: &Entity) {
        // Fire the event
        match self.entity_added {
            Some(entity_added_evn) => entity_added_evn(&mut self.custom_data, entity),
            None => {}
        }
    }
    // Remove an entity from the current system
    pub fn remove_entity(&mut self, entity: &Entity) {
        // Fire the event
        match self.entity_removed {
            Some(entity_removed_evn) => entity_removed_evn(&mut self.custom_data, entity),
            None => {}
        }
    }
    // Stop the system permanently
    pub fn end_system(&mut self, entities: &Vec<&Entity>) {
        match self.entity_removed {
            Some(entity_removed) => {
                // Fire the entity removed event
                for entity in entities.iter() {
                    entity_removed(&mut self.custom_data, entity);
                }
            }
            None => {}
        }
    }
    // Run the system for a single iteration
    pub fn run_system(&mut self, entities: &Vec<&Entity>) {
        // Pre fire event
        match self.system_prefire {
            Some(system_prefire_event) => system_prefire_event(&mut self.custom_data),
            None => {}
        }

        // Only update the entities if we have a a valid event. No need to waste time updating them ¯\_(ツ)_/¯
        match self.entity_update {
            Some(entity_update) => {
                // Loop over all the entities and fire the event
                for entity in entities.iter() {
                    entity_update(&mut self.custom_data, entity);
                }
            }
            None => {}
        }

        // Post fire event
        match self.system_postfire {
            Some(system_postfire) => system_postfire(&mut self.custom_data),
            None => {}
        }
    }
}

// Trait for custom system data
pub trait CustomSystemData {}

impl CustomSystemData for () {}
