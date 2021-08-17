use std::any::Any;

use super::{
    entity::Entity,
    system_data::{SystemData, SystemEventData, SystemEventDataLite, SystemState, SystemType},
};
use crate::engine::core::world::{Time};

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
    pub fn remove_entity_from_systems(
        &mut self,
        removed_entity: Entity,
        entity_id: u16,
        data: &mut SystemEventDataLite,
    ) {
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
        println!("Add system with cBitfield: {}", system_data.c_bitfield);
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
        for system in self
            .systems
            .iter_mut()
            .filter(|x| match x.get_system_data().stype {
                _system_type => true,
                _ => false,
            })
        {
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
    pub fn get_system_mut(&mut self, system_id: u8) -> &mut Box<dyn System> {
        let system = self.systems.get_mut(system_id as usize).unwrap();
        system
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
            println!(
                "\x1b[32mAdd entity '{}' with entity ID: {}, to the system '{}'\x1b[0m",
                entity.name, entity.entity_id, system_data.system_id
            );
            system_data.entities.push(entity.entity_id);
        }
        self.entity_added(entity, data);
    }
    // Remove an entity from the current system
    // NOTE: The entity was already removed in the world global entities, so the "removed_entity" argument is just the clone of that removed entity
    fn remove_entity(
        &mut self,
        entity_id: u16,
        removed_entity: &Entity,
        data: &mut SystemEventDataLite,
    ) {
        let system_data = self.get_system_data_mut();
        // Search for the entity with the matching entity_id
        let system_entity_id = system_data
            .entities
            .iter()
            .position(|&entity_id_in_vec| entity_id_in_vec == entity_id)
            .unwrap();
        system_data.entities.remove(system_entity_id);
        println!(
            "\x1b[33mRemoved entity '{}' with entity ID: {}, from the system '{}'\x1b[0m",
            removed_entity.name, removed_entity.entity_id, system_data.system_id
        );
        self.entity_removed(removed_entity, data);
    }
    // Stop the system permanently
    fn end_system(&mut self, data: &mut SystemEventDataLite) {
        let system_data_clone = self.get_system_data().clone();
        // Loop over all the entities and fire the entity removed event
        for &entity_id in system_data_clone.entities.iter() {
            let entity_clone = &mut data.entity_manager.get_entity(entity_id).unwrap().clone();
            self.entity_removed(entity_clone, data);
            *data.entity_manager.get_entity_mut(entity_id).unwrap() = entity_clone.clone();
        }
        *self.get_system_data_mut() = system_data_clone;
    }
    // Run the system for a single iteration
    fn run_system(&mut self, data: &mut SystemEventData) {
        let system_data_clone = self.get_system_data().clone();
        self.pre_fire(data);
        // Loop over all the entities and update their components
        for &entity_id in system_data_clone.entities.iter() {
            let mut entity_clone = data
                .entity_manager
                .get_entity_mut(entity_id)
                .unwrap()
                .clone();
            self.fire_entity(&mut entity_clone, data);
        }
        *self.get_system_data_mut() = system_data_clone;
        self.post_fire(data);
    }

    // Getters for the system data
    fn get_system_data(&self) -> &SystemData;
    fn get_system_data_mut(&mut self) -> &mut SystemData;

    // System Events
    fn entity_added(&mut self, _entity: &Entity, _data: &mut SystemEventDataLite) {}
    fn entity_removed(&mut self, _entity: &Entity, _data: &mut SystemEventDataLite) {}

    // System control functions
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData);
    fn pre_fire(&mut self, _data: &mut SystemEventData) {}
    fn post_fire(&mut self, _data: &mut SystemEventData) {}

    // As any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
