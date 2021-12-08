use crate::{Entity, ECSError, ComponentManager, ComponentID, EntityManager};

#[derive(Default)]
// Manages the systems, however each system is in it's own thread (For now at least)
pub struct SystemManager {
    pub systems: Vec<SystemThreadData>,
}

// Contains some data about the actual system on the thread
pub struct SystemThreadData {
}

// A system event enum
pub enum SystemEventType<T> {
    // Control events
    SystemEnabled(fn(&mut T)),
    SystemDisabled(fn(&mut T)),
    SystemPrefire(fn(&mut T)),
    SystemPostfire(fn(&mut T)),
    // Entity events
    EntityAdded(fn(&mut T, &Entity)),
    EntityRemoved(fn(&mut T, &Entity)),
    EntityUpdate(fn(&mut T, &Entity)),
    // Entity custom event
    EntityFilter(fn(&mut T, &Entity) -> bool),
}

// A system, stored on the stack, but it's SystemData is a trait object
#[derive(Default)]
pub struct System<T> where T: CustomSystemData {
    custom_data: T,
    c_bitfield: usize,
    system_id: u8,
    enabled: bool,
    entities: Vec<usize>,


    // Events
    // Control events
    system_enabled: Option<fn(&mut T)>,
    system_disabled: Option<fn(&mut T)>,
    system_prefire: Option<fn(&mut T)>,
    system_postfire: Option<fn(&mut T)>,
    // Entity events
    entity_added: Option<fn(&mut T, &Entity)>,
    entity_removed: Option<fn(&mut T, &Entity)>,
    entity_update: Option<fn(&mut T, &Entity)>,
    entity_filter: Option<fn(&mut T, &Entity) -> bool>,
}

// System code
impl<T> System<T> where T: CustomSystemData {
    // Check if a specified entity fits the criteria to be in a specific system
    fn is_entity_valid(&self, entity: &Entity) -> bool {
        // Check if the system matches the component ID of the entity
        let bitfield: usize = self.c_bitfield & !entity.c_bitfield;
        // If the entity is valid, all the bits would be 0
        bitfield == 0
    }
    // Add a component to this system's component bitfield id
    pub fn link_component<U: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        if crate::registry::is_component_registered::<U>() {
            // Link the component if it was already registered
            let c = crate::registry::get_component_id::<U>()?;
            self.c_bitfield |= c;
        } else {
            // Link the component if it was not registered yet
            self.c_bitfield |= crate::registry::register_component::<U>();
        }
        Ok(())
    }
    // Attach the a specific system event
    pub fn event(&mut self, event: SystemEventType<T>) {
        match event {
            // Control events
            SystemEventType::SystemEnabled(x) => self.system_enabled = Some(x),
            SystemEventType::SystemDisabled(x) => self.system_disabled = Some(x),
            SystemEventType::SystemPrefire(x) => self.system_prefire = Some(x),
            SystemEventType::SystemPostfire(x) => self.system_postfire = Some(x),
            // Entity events
            SystemEventType::EntityAdded(x) => self.entity_added = Some(x),
            SystemEventType::EntityRemoved(x) => self.entity_removed = Some(x),
            SystemEventType::EntityUpdate(x) => self.entity_update = Some(x),
            SystemEventType::EntityFilter(x) => self.entity_filter = Some(x),
        };
    }
    // Add an entity to the current system
    fn add_entity(&mut self, entity: &Entity) {
        if !self.enabled {
            return;
        }
        self.entities.push(entity.entity_id);
        // Fire the event
        match self.entity_added {
            Some(entity_added_evn) => entity_added_evn(&mut self.custom_data, entity),
            None => {}
        }
    }
    // Remove an entity from the current system
    fn remove_entity(&mut self, entity_id: usize, entity: &Entity) {
        if !self.enabled {
            return;
        }
        // Search for the entity with the matching entity_id
        let system_entity_local_id = self.entities.iter().position(|&entity_id_in_vec| entity_id_in_vec == entity_id).unwrap();
        self.entities.remove(system_entity_local_id);
        // Fire the event
        match self.entity_removed {
            Some(entity_removed_evn) => entity_removed_evn(&mut self.custom_data, entity),
            None => {}
        }
    }
    // Stop the system permanently
    fn end_system(&mut self, entity_manager: &EntityManager) {
        match self.entity_removed {
            Some(x) => {
                // Fire the entity removed event
                for entity_id in self.entities.iter() {
                    /*
                    // Get the entity
                    let entity = entity_manager.get_entity(*entity_id).unwrap().clone();
                    x(&entity);
                    */
                }
            }
            None => {}
        }
    }
    // Run the system for a single iteration
    fn run_system(&mut self, entity_manager: &EntityManager) {
        if !self.enabled {
            return;
        }
        // Pre fire event
        match self.system_prefire {
            Some(system_prefire_event) => system_prefire_event(&mut self.custom_data),
            None => {}
        }

        // Filter the entities
        let entity_manager_immutable = entity_manager;
        let filtered_entity_ids = match self.entity_filter {
            Some(x) => {
                self.entities
                    .iter()
                    .filter(|entity_id| {
                        /*
                        // Filter the entities
                        let entity_clone = &entity_manager_immutable.get_entity(**entity_id).unwrap();
                        // Get the linked components
                        let linked_components = LinkedComponents::get_linked_components(entity_clone, self.flc_c_bitfield);
                        x(&linked_components)
                        */
                        true
                    })
                    .cloned()
                    .collect()
            }
            None => {
                // No filtering, just return all the entities
                self.entities.clone()
            }
        };
        // Only update the entities if we have a a valid event. No need to waste time updating them ¯\_(ツ)_/¯
        match self.entity_update {
            Some(x) => {
                // Loop over all the entities and fire the event
                for entity_id in filtered_entity_ids {
                    /*
                    let entity_clone = entity_manager.get_entity(entity_id).unwrap().clone();
                    // Get the linked entity components from the current entity
                    let linked_components = LinkedComponents::get_linked_components(&entity_clone, self.flc_c_bitfield);
                    x(&entity_clone, &linked_components);
                    */
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
    // Enable this system
    pub fn enable(&mut self) {
        self.enabled = true;
        // Fire the event
        match self.system_enabled {
            Some(system_enabled) => system_enabled(&mut self.custom_data),
            None => {}
        }
    }
    // Disable this system
    pub fn disable(&mut self) {
        self.enabled = false;
        // Fire the event
        match self.system_disabled {
            Some(system_disabled) => system_disabled(&mut self.custom_data),
            None => {}
        }
    }
}

// Trait for custom system data 
pub trait CustomSystemData {
}