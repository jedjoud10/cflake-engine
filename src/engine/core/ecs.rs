use crate::engine::{core::world::World, input::InputManager, rendering::{shader::ShaderManager, texture::TextureManager}};
use std::{any::Any, collections::HashMap, hash::Hash};

use super::world::{EntityManager, Time};

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u16 = 16;

// A component trait that can be added to other components
pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Struct used to get the component ID of specific components, entities, and systems
#[derive(Default)]
pub struct ComponentManager {
    pub component_ids: HashMap<String, u16>,
    pub components: Vec<Box<dyn Component>>,
    pub discrete_components: Vec<Box<dyn Component>>,
    pub current_component_id: u16,
}

// Implement all the functions
impl ComponentManager {
	// Gets a mutable reference to a component using it's component ID
    // Registers a specific component
    pub fn register_component<T: ComponentID>(&mut self) -> u16 {
        let name: String = T::get_component_name();
        // Register the component
        self.component_ids
            .insert(name.clone(), self.current_component_id);
        // Make a copy of the id before the bit shift
        let component_id = self.current_component_id;
        // Bit shift to the left
        self.current_component_id = self.current_component_id << 1;
        // Return the component id before the bit shift
        println!("Registered component '{}' with ID {}", name, component_id);
        component_id
    }

    // Get the component id for a specific entity
    pub fn get_component_id<T: ComponentID>(&self) -> u16 {
        let name: String = T::get_component_name();
        // It found the component, so just return it's id
        if self.component_ids.contains_key(&name) {
            let value = self.component_ids[&name];
            return value;
        } else {
            panic!("Component {} not registered!", name);
        }
    }

    // Checks if a specific component is registered
    pub fn is_component_registered<T: ComponentID>(&self) -> bool {
        self.component_ids.contains_key(&T::get_component_name())
    }

    // Get the component id for a specific entity
    pub fn get_component_id_by_name(&self, name: &String) -> u16 {
        // It found the component, so just return it's id
        if self.component_ids.contains_key(name) {
            let value = self.component_ids[name];
            return value;
        } else {
            panic!("Component {} not registered!", name);
        }
    }

	// Add a discrete component to the world, that isn't linked to any entity
    pub fn add_discrete_component<'a, T: ComponentID + Component + 'static>(
        &mut self,
        component: T,
    ) -> u16 {
        // Make sure the component is registered first
        if !self.is_component_registered::<T>() {
            self.register_component::<T>();
        }
        // Add the component, and return it's id
        self
            .discrete_components
            .push(Box::new(component));
        let index = self.discrete_components.len() as u16 - 1;
        return index;
    }
    // Get a reference to a specific discrete component from the world, without the need of an entity
    pub fn get_dicrete_component<'a, T: ComponentID + Component + 'static>(
        &self,
        index: u16,
    ) -> &T {
        let component_any = self
            .discrete_components
            .get(index as usize)
            .unwrap()
            .as_any();
        let component: &T = component_any.downcast_ref().unwrap();
        return component;
    }
}

// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}

// Tells you the state of the system, and for how long it's been enabled/disabled
#[derive(Clone, Copy)]
pub enum SystemState {
    Enabled(f32),
    Disabled(f32),
}

// All of the systems that are implement by default
#[derive(Clone, Copy)]
pub enum SystemType {
    // Main System Types: Used for scripting
    Update,
    Tick,
    Render,

    // Additional Default System: Uses the main system types
    Physics,
    GUI,
    Terrain,
}

// Some system data that is part of a system and wrapped around System trait getter functions
#[derive(Clone)]
pub struct SystemData {
    pub c_bitfield: u16,
    pub system_id: u8,
    pub state: SystemState,
    pub stype: SystemType,
    pub entities: Vec<u16>,
    pub system_components: HashMap<u16, u16>,
}

// Default for system data
impl Default for SystemData {
    fn default() -> Self {
        Self {
            c_bitfield: 0,
            system_id: 0,
            state: SystemState::Enabled(0.0),
            stype: SystemType::Update,
            entities: Vec::new(),
            system_components: HashMap::new(),
        }
    }
}

impl SystemData {
    // Add a component to this system's component bitfield id
    pub fn link_component<T: ComponentID>(&mut self, world: &mut World) {
        if world.component_manager.is_component_registered::<T>() {
            self.c_bitfield = self.c_bitfield | world.component_manager.get_component_id::<T>();
        } else {
            world.component_manager.register_component::<T>();
            self.c_bitfield = self.c_bitfield | world.component_manager.get_component_id::<T>();
        }
        println!(
            "Link component '{}' to system '{}', with ID: {}",
            T::get_component_name(),
            self.system_id,
            world.component_manager.get_component_id::<T>()
        );
    }
}

// Data that will be passed to the fire events in systems
pub struct FireData<'a> {
	pub entity_manager: &'a mut EntityManager,
	pub component_manager: &'a mut ComponentManager,
	pub input_manager: &'a mut InputManager,
	pub shader_manager: &'a mut ShaderManager,
	pub texture_manager: &'a mut TextureManager,
	pub time_manager: &'a mut Time,
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct FireDataFragment<'a> {
	pub entity_manager: &'a mut EntityManager,
	pub component_manager: &'a mut ComponentManager
}

pub trait System {	
	// Setup the system, link all the components and create default data 
	fn setup_system(&mut self, world: &mut World);
	// Add an entity to the current system
	fn add_entity(&mut self, entity: &Entity, world: &mut World) {
		let system_data = self.get_system_data_mut();
        println!(
            "\x1b[32mAdd entity '{}' with entity ID: {}, to the system '{}'\x1b[0m",
            entity.name, entity.entity_id, system_data.system_id
        );
        system_data.entities.push(entity.entity_id);
	}
	// Remove an entity from the current system
	// NOTE: The entity was already removed in the world global entities, so the "removed_entity" argument is just the clone of that removed entity
	fn remove_entity(&mut self, entity_id: u16, removed_entity: &Entity, world: &mut World) {
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
	}
	// Stop the system permanently
	fn end_system(&mut self, data: &mut FireDataFragment) {
		let system_data_clone = self.get_system_data().clone();
        // Loop over all the entities and fire the entity removed event
        for &entity_id in system_data_clone.entities.iter() {
            let entity_clone = &mut data.entity_manager.get_entity(entity_id).clone();
            self.entity_removed(entity_clone, data);
			*data.entity_manager.get_entity_mut(entity_id) = entity_clone.clone();
        }
		*self.get_system_data_mut() = system_data_clone;
    }
	// Run the system for a single iteration
	fn run_system(&mut self, data: &mut FireData) {
		let system_data_clone = self.get_system_data().clone();
		self.pre_fire(data);
        // Loop over all the entities and update their components
        for &entity_id in system_data_clone.entities.iter() {
            let mut entity_clone = data.entity_manager.get_entity_mut(entity_id).clone();
            self.fire_entity(&mut entity_clone, data);					
        }
		*self.get_system_data_mut() = system_data_clone;
        self.post_fire(data);
	}

    // Getters for the system data
	fn get_system_data(&self) -> &SystemData;
	fn get_system_data_mut(&mut self) -> &mut SystemData;

	// System Events
	fn entity_added(&mut self, entity: &Entity, data: &mut FireDataFragment) {

	}
	fn entity_removed(&mut self, entity: &Entity, data: &mut FireDataFragment) {

	}

	// System control functions
	fn fire_entity(&mut self, entity: &mut Entity, data: &mut FireData);
	fn pre_fire(&mut self, data: &mut FireData);
	fn post_fire(&mut self, data: &mut FireData);
}

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
        return bitfield == 0;
    }
	// Remove an entity from it's corresponding systems
	pub fn remove_entity_from_systems(&mut self, world: &mut World, removed_entity: Entity, entity_id: u16) {
		// Remove the entity from all the systems it was in
        for system in self.systems.iter_mut() {
            let system_data = system.get_system_data_mut();

            // Only remove the entity from the systems that it was in
            if Self::is_entity_valid_for_system(&removed_entity, system_data) {
                system.remove_entity(entity_id, &removed_entity, world);
            }
        }
	}
	// Add an entity to it's corresponding systems
	pub fn add_entity_to_systems(&mut self, entity: &Entity, world: &mut World) {
		// Check if there are systems that need this entity
        for system in self.systems.iter_mut() {
            let system_data = system.get_system_data_mut();
            if Self::is_entity_valid_for_system(&entity, system_data) {
                // Add the entity to the system
                system.add_entity(&entity, world);
            }
        }
	}
	// Add a system to the world, and returns it's system ID
	pub fn add_system(&mut self, system: Box<dyn System>) -> u16 {
		let id = self.systems.len() as u16;
		let system_data = system.get_system_data();
		println!("Add system with cBitfield: {}", system_data.c_bitfield);
		self.systems.push(system);
		return id;
	}
	// Kill all the systems
	pub fn kill_systems(&mut self, data: &mut FireDataFragment) {		
		for system in self.systems.iter_mut() {
			system.end_system(data);
		}		
	}
	// Runs a specific type of system
	pub fn run_system_type(&mut self, system_type: SystemType, data: &mut FireData) {
		for system in self.systems.iter_mut().filter(|x| 
			match x.get_system_data().stype {
    			SystemType::Update => return true,
				_ => return false
			}
		) {
			system.run_system(data);
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
                    *system_state =
                        SystemState::Disabled(*old_time + time.delta_time as f32);
                }
            }
        }
	}
	// Run a specific system, firing off the pre-fire, entity-fire, and post-fire events
	pub fn run_system(&mut self, system_id: u16, data: &mut FireData) {
		let system = self.systems.get_mut(system_id as usize).unwrap();
		system.run_system(data);
	}
	// Gets a reference to a system
	pub fn get_system(&self, system_id: u16) -> &Box<dyn System> {
		let system = self.systems.get(system_id as usize).unwrap();
		return system;
	}
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub c_bitfield: u16,
    // The actual components are stored in the world, this allows for two objects to share a single component if we want to have duplicate entities
    components: HashMap<u16, u16>,
}

// ECS time bois
impl Entity {
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default_component<T: ComponentID + Default + Component + 'static>(
        &mut self,
        world: &mut World,
    ) {
        let component_name = T::get_component_name();
        let component_id = world
            .component_manager
            .get_component_id_by_name(&component_name);
        world
            .component_manager
            .components
            .push(Box::new(T::default()));
        let world_component_id = world.component_manager.components.len() - 1;
        self.c_bitfield = self.c_bitfield | component_id;
        self.components
            .insert(component_id, world_component_id as u16);
        println!(
            "Link component '{}' to entity '{}', with ID: {} and global ID: '{}'",
            component_name, self.name, component_id, world_component_id
        );
    }
    // Link a component to this entity and use the gived default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        world: &mut World,
        default_state: T,
    ) {
        let component_name = T::get_component_name();
        let component_id = world
            .component_manager
            .get_component_id_by_name(&component_name);
        world
            .component_manager
            .components
            .push(Box::new(default_state));
        let world_component_id = world.component_manager.components.len() - 1;
        self.c_bitfield = self.c_bitfield | component_id;
        self.components
            .insert(component_id, world_component_id as u16);
        println!(
            "Link component '{}' to entity '{}', with ID: {} and global ID: '{}'",
            component_name, self.name, component_id, world_component_id
        );
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, world: &mut World) {
        let name = T::get_component_name();
        let id = world.component_manager.get_component_id_by_name(&name);
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield = (!id) & self.c_bitfield;
        self.components.remove(&id);
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> &'a T {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let component_any: &dyn Any = component_manager
                .components
                .get(self.components[&component_id] as usize)
                .unwrap()
                .as_any();
            let final_component = component_any.downcast_ref::<T>().unwrap();
            return final_component;
        } else {
            panic!(format!(
                "Component '{}' does not exist on entity '{}'!",
                T::get_component_name(),
                self.name
            ));
        }
    }
    // Gets a specific component, mutably
    pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a mut ComponentManager,
    ) -> &'a mut T {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let component_any: &mut dyn Any = component_manager
                .components
                .get_mut(self.components[&component_id] as usize)
                .unwrap()
                .as_any_mut();
            let final_component = component_any.downcast_mut::<T>().unwrap();
            return final_component;
        } else {
            panic!(format!(
                "Component '{}' does not exist on entity '{}'!",
                T::get_component_name(),
                self.name
            ));
        }
    }
}
