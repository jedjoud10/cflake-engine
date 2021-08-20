use std::any::Any;
use glfw::ffi::DECORATED;

use super::{
     component::{ComponentManager, FilteredLinkedComponents},
    entity::Entity,
    error::ECSError,
    system_data::{SystemData, SystemEventData, SystemEventDataLite, SystemState, SystemType},
};
use crate::engine::core::world::Time;

#[derive(Default)]
// Manages the systems
pub struct SystemManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    // Check if a specified entity fits the criteria to be in a specific system
    fn is_entity_valid_for_system(entity: &Entity, system_data: &SystemData) -> bool {
        // Check if the system matches the component ID of the entity
        let bitfield: u16 = system_data.c_bitfield & !entity.c_bitfield;
        // If the entity is valid, all the bits would be 0
        bitfield == 0
    }
    // Remove an entity from it's corresponding systems
    pub fn remove_entity_from_systems(&mut self, removed_entity: Entity, entity_id: u16, data: &mut SystemEventDataLite) {
        // Remove the entity from all the systems it was in
        for system in self.systems.iter_mut() {
            let system_data = system.get_system_data_mut();

            // Only remove the entity from the systems that it was in
            if Self::is_entity_valid_for_system(&removed_entity, system_data) {
                system.remove_entity(entity_id, &removed_entity, data);
            }
        }
    }
    // Add an entity to it's corresponding systems
    pub fn add_entity_to_systems(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // Check if there are systems that need this entity
        for system in self.systems.iter_mut() {
            let system_data = system.get_system_data_mut();
            if Self::is_entity_valid_for_system(entity, system_data) {
                // Add the entity to the system
                system.add_entity(entity, data);
            }
        }
    }
    // Add a system to the world, and returns it's system ID
    pub fn add_system<T: 'static + System>(&mut self, mut system: T) -> u8 {
        let id = self.systems.len() as u8;
        let system_data = system.get_system_data_mut();
        system_data.system_id = id;
        self.systems.push(Box::new(system));
        id
    }
    // Kill all the systems
    pub fn kill_systems(&mut self, data: &mut SystemEventDataLite) {
        for system in self.systems.iter_mut() {
            system.end_system(data);
        }
    }
    // Runs a specific type of system
    pub fn run_system_type(&mut self, _system_type: SystemType, data: &mut SystemEventData) {
        for system in self.systems.iter_mut().filter(|x| match x.get_system_data().stype {
            // TODO: Uhhh fix this
            _system_type => true,
            _ => false,
        }) {
            match system.get_system_data().state {
                SystemState::Enabled(_f) => {
                    system.run_system(data);
                }
                _ => {}
            }
        }
    }
    // Update system timings
    pub fn update_systems(&mut self, time: &Time) {
        for system in self.systems.iter_mut() {
            let system_state = &mut system.get_system_data_mut().state;
            match system_state {
                // Keep track of how many seconds the system's been enabled/disabled
                SystemState::Enabled(old_time) => {
                    *system_state = SystemState::Enabled(*old_time + time.delta_time as f32);
                }
                SystemState::Disabled(old_time) => {
                    *system_state = SystemState::Disabled(*old_time + time.delta_time as f32);
                }
            }
        }
    }
    // Gets a reference to a system
    pub fn get_system(&self, system_id: u8) -> &Box<dyn System> {
        let system = self.systems.get(system_id as usize).unwrap();
        system
    }
    // Gets a mutable reference to a system
    pub fn get_system_mut<'a, T: System + 'static>(&'a mut self, system_id: u8) -> Result<&'a mut T, ECSError> {
        let system = self
            .systems
            .get_mut(system_id as usize)
            .ok_or::<ECSError>(ECSError::new(format!("System with ID: '{}' does not exist!", system_id).as_str()))?;
        let cast_system = system
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or::<ECSError>(ECSError::new(format!("Could not cast system to type: '{}'!", std::any::type_name::<T>()).as_str()))?;
        Ok(cast_system)
    }
}

pub trait System {
    // Setup the system, link all the components and create default data
    fn setup_system(&mut self, data: &mut SystemEventData);
    // When the system gets added the world
    fn system_added(&mut self, _data: &mut SystemEventData, _system_id: u8) {}
    // Add an entity to the current system
    fn add_entity(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        {
            let system_data = self.get_system_data_mut();
            system_data.entities.push(entity.entity_id);
        }
        self.entity_added(entity, data);
    }
    // Remove an entity from the current system
    // NOTE: The entity was already removed in the world global entities, so the "removed_entity" argument is just the clone of that removed entity
    fn remove_entity(&mut self, entity_id: u16, removed_entity: &Entity, data: &mut SystemEventDataLite) {
        let system_data = self.get_system_data_mut();
        // Search for the entity with the matching entity_id
        let system_entity_id = system_data.entities.iter().position(|&entity_id_in_vec| entity_id_in_vec == entity_id).unwrap();
        system_data.entities.remove(system_entity_id);
        self.entity_removed(removed_entity, data);
    }
    // Stop the system permanently
    fn end_system(&mut self, data: &mut SystemEventDataLite) {
        let system_data_clone = self.get_system_data_mut();
        let entities_clone = system_data_clone.entities.clone();
        // Loop over all the entities and fire the entity removed event
        for &entity_id in entities_clone.iter() {
            let entity_clone = &mut data.entity_manager.get_entity(entity_id).unwrap().clone();
            self.entity_removed(entity_clone, data);
            *data.entity_manager.get_entity_mut(entity_id).unwrap() = entity_clone.clone();
        }
        // Reput the cloned entities
        self.get_system_data_mut().entities = entities_clone;
    }
    // Run the system for a single iteration
    fn run_system(&mut self, data: &mut SystemEventData) {
        // Pre fire event call
        self.pre_fire(data);
        let system_data = self.get_system_data_mut();
        let entities_clone = system_data.entities.clone();
        let c_bitfield = system_data.c_bitfield;
        let invalid_entity_ppf = system_data.entity_ppf.as_ref();
        let valid_test = invalid_entity_ppf.unwrap();
        // Not yet initialized
        let mut valid_ppf: bool = false;
        match invalid_entity_ppf {
            None => valid_ppf = true,
            _ => { }
        }

        // Loop over all the entities and update their components
        for &entity_id in entities_clone.iter() {
            let entity_clone = data.entity_manager.get_entity_mut(entity_id).unwrap().clone();
            let test: bool = false;
            {
                let is_valid_entity = valid_test.filter_entity(&entity_clone);

            }
            // TODO: Make this actually filter and don't skip
            if test {
                // Get the linked entity components from the current entity
                let mut linked_entity_components = FilteredLinkedComponents::get_filtered_linked_components(&entity_clone, c_bitfield.clone());
                self.fire_entity(&mut linked_entity_components, data);
            } else {
                continue;
            }
        }
        // Reput the cloned entities
        self.get_system_data_mut().entities = entities_clone;
        // Post fire event call
        self.post_fire(data);
    }

    // Getters for the system data
    fn get_system_data(&self) -> &SystemData;
    fn get_system_data_mut(&mut self) -> &mut SystemData;

    // System Events
    fn entity_added(&mut self, _entity: &Entity, _data: &mut SystemEventDataLite) {}
    fn entity_removed(&mut self, _entity: &Entity, _data: &mut SystemEventDataLite) {}

    // System control functions
    fn generate_eppf(&mut self) -> Box<dyn EntityPrePassFilter> {
        let eppf = DefaultEntityPrePassFilter::default();
        return Box::new(eppf);
    }
    fn fire_entity(&mut self, components: &mut FilteredLinkedComponents, data: &mut SystemEventData);
    fn pre_fire(&mut self, _data: &mut SystemEventData) {}
    fn post_fire(&mut self, _data: &mut SystemEventData) {}

    // As any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Pre pass filter for the entities
pub trait EntityPrePassFilter {
    fn filter_entity(&self, entity: &Entity) -> bool;
}

// The default entity pre pass filter
#[derive(Default)]
pub struct DefaultEntityPrePassFilter {
}

// The default entity pre pass filter, so make every entity pass the filter succsesfully
impl EntityPrePassFilter for DefaultEntityPrePassFilter {
    fn filter_entity(&self, entity: &Entity) -> bool {
        true
    }
}