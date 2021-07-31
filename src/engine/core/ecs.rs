use std::{any::Any, collections::HashMap, hash::Hash};
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
#[derive(Clone)]
pub struct System {
	pub name: String,
	pub c_bitfield: u8,
	pub system_id: u8,
	pub state: SystemState,
	pub stype: SystemType,
	// Entity events
	pub entity_loop_event: fn(&Entity, &mut World),
	pub entity_added_event: fn(&Entity, &mut World),
	pub entity_removed_event: fn(&Entity, &mut World),

	pub entities: Vec<u16>,
	pub system_components: HashMap<u8, u16>,
}

// Default for system data
impl Default for System {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed system"),
			c_bitfield: 0,
			system_id: 0,
			state: SystemState::Enabled(0.0),
			stype: SystemType::Update,
			entity_loop_event: |_entity, _world| {},
			entity_added_event: |_entity, _world|  {},
			entity_removed_event: |_entity, _world|  {},
			entities: Vec::new(),
			system_components: HashMap::new()
		}
	}
}

impl System {
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
			*world.get_entity_mut(entity_id) = entity_clone.clone();
		}
	}
	// Fire the "entity_loop" event
	pub fn run_entity_loops(&mut self, world: &mut World) {
		// Loop over all the entities and update their components
		for &entity_id in self.entities.iter() {		
			let entity_clone = &mut world.get_entity(entity_id).clone();
			(self.entity_loop_event)(entity_clone, world);
			*world.get_entity_mut(entity_id) = entity_clone.clone();
		}
	}
	// Add a component to this system's component bitfield id
	pub fn link_component<T: ComponentID>(&mut self, world: &mut World) {
		if world.component_manager.is_component_registered::<T>() {
			self.c_bitfield = self.c_bitfield | world.component_manager.get_component_id::<T>();			
		} else {
			world.component_manager.register_component::<T>();
			self.c_bitfield = self.c_bitfield | world.component_manager.get_component_id::<T>();
		}		
		println!("Link component '{}' to system '{}', with ID: {}", T::get_component_name(), self.name, world.component_manager.get_component_id::<T>());
	}
	// Add a SystemComponent; a custom type of component that is just for systems
	pub fn link_system_component<T: SystemComponent + ComponentID + Default + 'static>(&mut self, world: &mut World) {
		// Check if we have the component already added
		let component_id = world.component_manager.get_component_id::<T>(); 
		if !self.system_components.contains_key(&component_id) {
			world.system_components.push(Box::new(T::default()));
			let global_component_id: u16 = world.system_components.len() as u16 - 1;
			self.system_components.insert(component_id, global_component_id);
		}
	}
	// Gets a reference to a system component
	pub fn get_system_component<'a, T: SystemComponent + ComponentID + 'static>(&self, world: &'a World) -> &'a T {
		let component_id = world.component_manager.get_component_id::<T>();
		// Check if we even have the component
		if self.system_components.contains_key(&component_id) {
			let component_any: &dyn Any = world.system_components.get(self.system_components[&component_id] as usize).unwrap().as_any();
			let final_component = component_any.downcast_ref::<T>().unwrap();
			return final_component;
		} else {
			panic!(format!("Component '{}' does not exist on entity '{}'!", T::get_component_name(), self.name));
		}
	}
	// Gets a mutable reference ot a system component
	pub fn get_system_component_mut<'a, T: SystemComponent + ComponentID + 'static>(&self, world: &'a mut World) -> &'a mut T {
		let component_id = world.component_manager.get_component_id::<T>();
		// Check if we even have the component
		if self.system_components.contains_key(&component_id) {
			let component_any: &mut dyn Any = world.system_components.get_mut(self.system_components[&component_id] as usize).unwrap().as_any_mut();
			let final_component = component_any.downcast_mut::<T>().unwrap();
			return final_component;
		} else {
			panic!(format!("Component '{}' does not exist on entity '{}'!", T::get_component_name(), self.name));
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
		println!("\x1b[33mRemoved entity '{}' with entity ID: {}, from the system '{}'\x1b[0m", removed_entity.name, removed_entity.entity_id, self.name);
	}
}

// A custom component type for systems
pub trait SystemComponent {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
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
	// Link a component to this entity and automatically set it to the default variable
	pub fn link_default_component<T: ComponentID + Default + Component + 'static>(&mut self, world: &mut World) {
		let component_name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&component_name);
		world.component_manager.components.push(Box::new(T::default()));
		let world_component_id = world.component_manager.components.len() - 1;
		self.c_bitfield = self.c_bitfield | component_id;
		self.components.insert(component_id, world_component_id as u16);
		println!("Link component '{}' to entity '{}', with ID: {} and global ID: '{}'", component_name, self.name, component_id, world_component_id);
	}
	// Link a component to this entity and use the gived default state parameter
	pub fn link_component<T: ComponentID + Component + 'static>(&mut self, world: &mut World, default_state: T) {
		let component_name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&component_name);
		world.component_manager.components.push(Box::new(default_state));
		let world_component_id = world.component_manager.components.len() - 1;
		self.c_bitfield = self.c_bitfield | component_id;
		self.components.insert(component_id, world_component_id as u16);
		println!("Link component '{}' to entity '{}', with ID: {} and global ID: '{}'", component_name, self.name, component_id, world_component_id);
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
	pub fn get_component<'a, T: ComponentID + Component + 'static>(&self, world: &'a World) -> &'a T {		
		let component_id = world.component_manager.get_component_id::<T>();
		// Check if we even have the component
		if self.components.contains_key(&component_id) {
			let component_any: &dyn Any = world.component_manager.components.get(self.components[&component_id] as usize).unwrap().as_any();
			let final_component = component_any.downcast_ref::<T>().unwrap();
			return final_component;
		} else {
			panic!(format!("Component '{}' does not exist on entity '{}'!", T::get_component_name(), self.name));
		}
		
	}
	// Gets a specific component, mutably
	pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(&self, world: &'a mut World) -> &'a mut T {
		let component_id = world.component_manager.get_component_id::<T>();
		// Check if we 
		// Check if we even have the component
		if self.components.contains_key(&component_id) {
			let component_any: &mut dyn Any = world.component_manager.components.get_mut(self.components[&component_id] as usize).unwrap().as_any_mut();
			let final_component = component_any.downcast_mut::<T>().unwrap();
			return final_component;
		} else {
			panic!(format!("Component '{}' does not exist on entity '{}'!", T::get_component_name(), self.name));
		}
	}
}