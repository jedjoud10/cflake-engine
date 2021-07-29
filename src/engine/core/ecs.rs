use std::{any::Any, collections::HashMap};
use crate::engine::core::world::World;

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
	pub component_ids: HashMap<String, u8>,	
	pub components: Vec<Box<dyn Component>>,
	pub current_component_id: u8
}

// Implement all the functions
impl ComponentManager {

	// Registers a specific component
	pub fn register_component<T: ComponentID>(&mut self) -> u8 {
		let name: String = T::get_component_name();	
		// Register the component
		self.component_ids.insert(name.clone(), self.current_component_id);
		// Make a copy of the id before the bit shift
		let component_id = self.current_component_id;		
		// Bit shift to the left
		self.current_component_id = self.current_component_id << 1;		
		// Return the component id before the bit shift
		println!("Registered component '{}' with ID {}", name, component_id);
		component_id
	}

	// Get the component id for a specific entity
	pub fn get_component_id<T: ComponentID>(&self) -> u8 {
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
	pub fn get_component_id_by_name(&self, name: &String) -> u8 {
		// It found the component, so just return it's id
		if self.component_ids.contains_key(name) {
			let value = self.component_ids[name];
			return value;
		}
		else {
			panic!("Component {} not registered!", name);
		}
	}
}

// A trait used to identify each component by their name
pub trait ComponentID {	
	fn get_component_name() -> String;
}

// Tells you the state of the system, and for how long it's been enabled/disabled
#[derive(Clone)]
pub enum SystemState {
	Enabled(f32),
	Disabled(f32)
}

// All of the systems that are implement by default
#[derive(Clone)]
pub enum SystemType {
	// Main System Types: Used for scripting
	Update,
	Tick,
	Render,

	// Additional Default System: Uses the main system types
	Physics,
	GUI,
	Terrain
}

// A generic system that can be used in 3 different ways (Tick system, Update system, Render system)
#[derive(Default, Clone)]
pub struct System {
	pub system_data: SystemData,
}

// A system that can write/read component data, every frame, or at the start of the game
#[derive(Clone)]
pub struct SystemData {
	pub name: String,
	pub c_bitfield: u8,
	pub system_id: u8,
	pub state: SystemState,
	pub stype: SystemType,
	// System events
	pub loop_event: fn(&World),
	// Entity events
	pub entity_loop_event: fn(&Entity, &mut World),
	pub entity_added_event: fn(&Entity, &mut World),
	pub entity_removed_event: fn(&Entity, &mut World),

	pub entities: Vec<u16>,
}

// Default for system data
impl Default for SystemData {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed system"),
			c_bitfield: 0,
			system_id: 0,
			state: SystemState::Enabled(0.0),
			stype: SystemType::Update,
			loop_event:  |world| {},
			entity_loop_event: |entity, world| {},
			entity_added_event: |entity, world|  {},
			entity_removed_event: |entity, world|  {},
			entities: Vec::new(),
		}
	}
}

impl SystemData {
	// Basic control code
	pub fn system_addded(&mut self) {

	}
	// Enable this current system
	pub fn enable_system(&mut self) {
		self.state = SystemState::Enabled(0.0);
	}
	// Disable the system and stop it from updating
	pub fn disable_system(&mut self) {
		self.state = SystemState::Disabled(0.0);
	}
	// End the system since the world is stopping
	pub fn end_system(&mut self, world: &mut World) {
		// Loop over all the entities and fire the entity removed event
		for &entity_id in self.entities.iter() {		
			let entity_clone = &mut world.get_entity(entity_id).clone();
			(self.entity_removed_event)(entity_clone, world);
			*world.get_entity(entity_id) = entity_clone.clone();
		}
	}
	// Fire the "entity_loop" event
	pub fn run_entity_loops(&mut self, world: &mut World) {
		// Loop over all the entities and update their components
		for &entity_id in self.entities.iter() {		
			let entity_clone = &mut world.get_entity(entity_id).clone();
			(self.entity_loop_event)(entity_clone, world);
			*world.get_entity(entity_id) = entity_clone.clone();
		}
	}
	// Add a component to this system's component bitfield id
	pub fn link_component<T: ComponentID>(&mut self, world: &mut World) {
		if world.component_manager.is_component_registered::<T>() {
			self.c_bitfield = self.c_bitfield | world.component_manager.get_component_id::<T>();			
		} else {
			world.component_manager.register_component::<T>();
		}		
	}
	// Adds an entity to the system
	pub fn add_entity(&mut self, entity: &Entity, world: &mut World) {
		println!("\x1b[32mAdd entity '{}' with entity ID: {}, to the system '{}'\x1b[0m", entity.name, entity.entity_id, self.name);
		self.entities.push(entity.entity_id);
		(self.entity_added_event)(&entity, world);
	}
	// Removes an entity from the system
	pub fn remove_entity(&mut self, entity_id: u16, removed_entity: &Entity, world: &mut World) {
		// Search for the entity with the matching entity_id
		let system_entity_id = self.entities.iter().position(|&entity_id_in_vec| entity_id_in_vec == entity_id).unwrap();
		self.entities.remove(system_entity_id);
		(self.entity_removed_event)(&removed_entity, world);
		println!("\x1b[33mRemoved entity '{}' with entity ID {}, from the system '{}'\x1b[0m", removed_entity.name, removed_entity.entity_id, self.name);
	}
}

// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
	pub name: String,
	pub entity_id: u16,
	pub c_bitfield: u8,
	// The actual components are stored in the world, this allows for two objects to share a single component if we want to have duplicate entities
	components: HashMap<u8, u16>,
}

// ECS time bois
impl Entity {
	// Link a component to this entity
	pub fn link_component<T: ComponentID + Component + 'static>(&mut self, world: &mut World, component: T) {
		let component_name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&component_name);
		world.component_manager.components.push(Box::new(component));
		let world_component_id = world.component_manager.components.len() - 1;
		self.c_bitfield = self.c_bitfield | component_id;
		self.components.insert(component_id, world_component_id as u16);
		println!("Link component '{}' to entity '{}', with ID {}", component_name, self.name, component_id);
	}
	// Unlink a component from this entity
	pub fn unlink_component<T: ComponentID>(&mut self, world: &mut World) {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(&name);
		// Take the bit, invert it, then AND it to the bitfield
		self.c_bitfield = (!id) & self.c_bitfield;
		self.components.remove(&id);
	}
	// Gets a specific component
	pub fn get_component<'a, T: ComponentID + Component + 'static>(&'a self, world: &'a  World) -> &'a T {
		let name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&name);

		let entity_component_id = self.components[&component_id];
		let final_component = &world.component_manager.components[entity_component_id as usize];
		let output_component = final_component.as_any().downcast_ref::<T>().expect("Component mismatch!");
		output_component
	}
	// Gets a specific component, mutably
	pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(&'a self, world: &'a mut World) -> &'a mut T {
		let name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&name);

		let entity_component_id = self.components[&component_id];
		let final_component = &mut world.component_manager.components[entity_component_id as usize];
		let output_component = final_component.as_any_mut().downcast_mut::<T>().expect("Component mismatch!");
		output_component
	}
}