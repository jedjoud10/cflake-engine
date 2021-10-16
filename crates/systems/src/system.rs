use ecs::*;
use errors::ECSError;
use others::Time;
use world_data::*;

use crate::{InternalSystemData, SystemData};

#[derive(Default)]
// Manages the systems
pub struct SystemManager {
    pub systems: Vec<System>,
}

impl SystemManager {    
    // Remove an entity from it's corresponding systems, this is done before actually removing the entity to allow the systems to dispose of it's data
    pub fn remove_entity_from_systems(&mut self, entity: &Entity, entity_id: usize, data: &mut WorldData) {
        // Remove the entity from all the systems it was in
        for system in self.systems.iter_mut() {
            // Only remove the entity from the systems that it was in
            if system.is_entity_valid(entity) {
                system.remove_entity(entity_id, entity, data);
            }
        }
    }
    // Add an entity to it's corresponding systems
    pub fn add_entity_to_systems(&mut self, entity: &Entity, data: &mut WorldData) {
        // Check if there are systems that need this entity
        for system in self.systems.iter_mut() {
            if system.is_entity_valid(entity) {
                // Add the entity to the system
                system.add_entity(entity, data);
            }
        }
    }
    // Add a system to the world, and returns it's system ID
    pub fn add_system(&mut self, mut system: System) -> u8 {
        let id = self.systems.len() as u8;
        system.system_id = id;
        self.systems.push(system);
        id
    }
    // Update all the systems
    pub fn update_systems(&mut self, data: &mut WorldData) {
        for system in self.systems.iter_mut() {
            system.run_system(data);
        }
    }
    // Kill all the systems
    pub fn kill_systems(&mut self, data: &mut WorldData) {
        for system in self.systems.iter_mut() {
            // Shut down each system first
            system.disable(data);
        }
        // Then end them
        for system in self.systems.iter_mut() {
            system.end_system(data);
        }
    }
    // Gets a reference to a system
    pub fn get_system(&self, system_id: u8) -> Result<&System, ECSError> {
        self.systems.get(system_id as usize).ok_or(ECSError::new_str("System does not exist!"))
    }
    // Gets a mutable reference to a system
    pub fn get_system_mut(&mut self, system_id: u8) -> Result<&mut System, ECSError> {
        self.systems.get_mut(system_id as usize).ok_or(ECSError::new_str("System does not exist!"))
    }
    // Gets a reference to the custom data of a specific system
    pub fn get_custom_system_data<T: InternalSystemData + 'static>(&self, system_id: u8) -> Result<&T, ECSError> {
        let system = self.get_system(system_id)?;
        let data = system.system_data.cast::<T>().unwrap();
        return Ok(data);
    }
    // Gets a mutable reference to the custom data a specific system 
    pub fn get_custom_system_data_mut<T: InternalSystemData + 'static>(&mut self, system_id: u8) -> Result<&mut T, ECSError> {
        let system = self.get_system_mut(system_id)?;
        let data = system.system_data.cast_mut::<T>().unwrap();
        return Ok(data);
    }
}
// A system event enum
pub enum SystemEventType {
    // Control events
    SystemEnabled(fn(&mut SystemData, &mut WorldData)),
    SystemDisabled(fn(&mut SystemData, &mut WorldData)),
    SystemPrefire(fn(&mut SystemData, &mut WorldData)),
    SystemPostfire(fn(&mut SystemData, &mut WorldData)),
    // Entity events
    EntityAdded(fn(&Entity, &mut WorldData)),
    EntityRemoved(fn(usize, &mut WorldData)),
    EntityUpdate(fn(&Entity, &FilteredLinkedComponents, &mut WorldData)),
}

// A system, stored on the stack, but it's SystemData is a trait object
#[derive(Default)]
pub struct System {
    c_bitfield: usize,
    system_id: u8,
    enabled: bool,
    entities: Vec<usize>,
    // The system data
    system_data: SystemData,

    // Events
    // Control events
    system_enabled_evn: Option<fn(&mut SystemData, &mut WorldData)>,
    system_disabled_evn: Option<fn(&mut SystemData, &mut WorldData)>,
    system_prefire_evn: Option<fn(&mut SystemData, &mut WorldData)>,
    system_postfire_evn: Option<fn(&mut SystemData, &mut WorldData)>,
    // Entity events
    entity_added_evn: Option<fn(&Entity, &mut WorldData)>,
    entity_removed_evn: Option<fn(usize, &mut WorldData)>,
    entity_update_evn: Option<fn(&Entity, &FilteredLinkedComponents, &mut WorldData)>,
}

// System code
impl System {
    // Create a simple system
    pub fn new() -> Self {
        Self {
            ..Self::default()
        }
    }
    // Check if a specified entity fits the criteria to be in a specific system
    fn is_entity_valid(&self, entity: &Entity) -> bool {
        // Check if the system matches the component ID of the entity
        let bitfield: usize = self.c_bitfield & !entity.c_bitfield;
        // If the entity is valid, all the bits would be 0
        bitfield == 0
    }
    // Add a component to this system's component bitfield id
    pub fn link_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        if component_manager.is_component_registered::<T>() {
            self.c_bitfield |= component_manager.get_component_id::<T>()?;
        } else {
            component_manager.register_component::<T>();
            self.c_bitfield |= component_manager.get_component_id::<T>()?;
        }
        Ok(())
    }
    // Attach the a specific system event
    pub fn event(&mut self, event: SystemEventType) {
        match event {
            // Control events
            SystemEventType::SystemEnabled(x) => self.system_enabled_evn = Some(x),
            SystemEventType::SystemDisabled(x) => self.system_disabled_evn = Some(x),
            SystemEventType::SystemPrefire(x) => self.system_prefire_evn = Some(x),
            SystemEventType::SystemPostfire(x) => self.system_postfire_evn = Some(x),
            // Entity events
            SystemEventType::EntityAdded(x) => self.entity_added_evn = Some(x),
            SystemEventType::EntityRemoved(x) => self.entity_removed_evn = Some(x),
            SystemEventType::EntityUpdate(x) => self.entity_update_evn = Some(x),
        };
    }
    // Add an entity to the current system
    fn add_entity(&mut self, entity: &Entity, data: &mut WorldData) {
        self.entities.push(entity.entity_id);
        // Fire the event
        match self.entity_added_evn {
            Some(x) => x(entity, data),
            None => {},
        }
    }
    // Remove an entity from the current system
    fn remove_entity(&mut self, entity_id: usize, entity: &Entity, data: &mut WorldData) {
        // Search for the entity with the matching entity_id
        let system_entity_local_id = self.entities.iter().position(|&entity_id_in_vec| entity_id_in_vec == entity_id).unwrap();
        let entity = self.entities.remove(system_entity_local_id);
        // Fire the event
        match self.entity_removed_evn {
            Some(x) => x(entity_id, data),
            None => {},
        }
    }
    // Stop the system permanently
    fn end_system(&mut self, data: &mut WorldData) {
        match self.entity_removed_evn {
            Some(x) => {
                // Fire the entity removed event
                for entity_id in self.entities.iter() {
                    (x)(*entity_id, data);
                }
            },
            None => {},
        }
    }
    // Run the system for a single iteration
    fn run_system(&mut self, data: &mut WorldData) {
        let entity_manager_immutable = &*data.entity_manager;
        // The filtered entities tuple that also contains the linked component data
        let filtered_entity_ids = self.entities
            .iter()
            .filter_map(|entity_id| {                
                let entity_clone = &entity_manager_immutable.get_entity(*entity_id).unwrap();
                // Get the linked components
                let linked_components = FilteredLinkedComponents::get_filtered_linked_components(entity_clone, self.c_bitfield);
                // Filtering the entities basically
                /*
                let valid_entity = self.filter_entity(entity_clone, &linked_components, &data);
                // Check if it is a valid entity
                if valid_entity {
                    // This entity passed the filter
                    Some(*entity_id)
                } else {
                    // This entity failed the filter
                    None
                }
                */
                Some(*entity_id)
            })
            .collect::<Vec<usize>>();

        // Only update the entities if we have a a valid event. No need to waste time updating them ¯\_(ツ)_/¯
        match self.entity_update_evn {
            Some(x) => {
                // Loop over all the entities and fire the event
                for entity_id in filtered_entity_ids {
                    let entity_clone = data.entity_manager.get_entity(entity_id).unwrap().clone();
                    // Get the linked entity components from the current entity
                    let linked_components = FilteredLinkedComponents::get_filtered_linked_components(&entity_clone, self.c_bitfield);
                    x(&entity_clone, &linked_components, data);
                }
            },
            None => {},
        }        
        
        // Post fire event
        match self.system_postfire_evn {
            Some(x) => x(&mut self.system_data, data),
            None => {},
        } 
    }
    // Enable this system
    pub fn enable(&mut self, data: &mut WorldData) {
        self.enabled = true;
        // Fire the event
        match self.system_enabled_evn {
            Some(x) => x(&mut self.system_data, data),
            None => {},
        }
    }
    // Disable this system
    pub fn disable(&mut self, data: &mut WorldData) {
        self.enabled = false;
        // Fire the event
        match self.system_disabled_evn {
            Some(x) => x(&mut self.system_data, data),
            None => {},
        }
    }
    // With custom data
    pub fn custom_data<T: InternalSystemData + 'static>(&mut self, data: T) {
        self.system_data.convert(data)
    }
}
