use std::{collections::HashMap};
use crate::engine::core::world::World;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u16 = 16;

// A component trait that can be added to other components
pub trait Component {
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
	pub component_ids: HashMap<String, u8>,	
	pub components: Vec<Box<dyn Component>>,
	pub current_component_id: u8
}

// Implement default values
impl Default for ComponentManager {
	fn default() -> Self { 
		Self {
			component_ids: HashMap::new(),
			components: Vec::new(),
			current_component_id: 1,
		}
	}
}

// Implement all the functions
impl ComponentManager {
	// Get the component id for a specific entity
	pub fn get_component_id<T: ComponentID>(&mut self) -> u8 {
		let name: String = T::get_component_name();
		
		// It found the component, so just return it's id
		if self.component_ids.contains_key(&name) {
			let value = self.component_ids[&name];
			return value;
		}
		// It did not find the component, so create a new "id binding" for one
		self.component_ids.insert(name, self.current_component_id);

		// Make a copy of the id before the bit shift
		let component_id = self.current_component_id;		
		// Bit shift to the left
		self.current_component_id = self.current_component_id << 1;
		// Return the component id before the bit shift
		component_id
	}

	// Get the component id for a specific entity
	pub fn get_component_id_by_name(&mut self, name: &String) -> u8 {
		// It found the component, so just return it's id
		if self.component_ids.contains_key(name) {
			let value = self.component_ids[name];
			return value;
		}
		
		// It did not find the component, so create a new "id binding" for one
		let name_val = String::from(name);
		self.component_ids.insert(name_val, self.current_component_id);

		// Make a copy of the id before the bit shift
		let component_id = self.current_component_id;		
		// Bit shift to the left
		self.current_component_id = self.current_component_id << 1;
		// Return the component id before the bit shift
		component_id
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
	pub system_data: SystemData,
	pub call_entity_event: fn(&Box<Entity>)
}

// A system that can write/read component data, every frame, or at the start of the game
#[derive(Clone)]
pub struct SystemData {
	pub name: String,
	pub component_bitfield: u8,
	pub system_id: u8,
	pub state: SystemState,
	pub stype: SystemType,
	pub entity_loop: fn(&Box<Entity>),
	pub entities: Vec<Box<Entity>>,
}

impl Default for SystemData {
	fn default() -> Self {
		let function = |entity: &Box<Entity>| {};
		Self {
			name: String::from("Unnamed system"),
			component_bitfield: 0,
			system_id: 0,
			state: SystemState::Disabled(0.0),
			stype: SystemType::Update,
			entity_loop: function,
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
	// Update the system
	pub fn update_system(&mut self) {
		// Loop over all the entities and update their components
		for entity in self.entities.iter() {		
			(self.entity_loop)(entity);
		}
	}
	// Add a component to this system's component bitfield id
	pub fn link_component<T: ComponentID>(&mut self, world: &mut World) {
		self.component_bitfield = self.component_bitfield | world.component_manager.get_component_id::<T>();
	}
	// Adds an entity to the system
	pub fn add_entity(&mut self, entity: Box<Entity>) {
		println!("Add entity '{}' with ID {}, to the system '{}'", entity.name, entity.entity_id, self.name);
		self.entities.push(entity);
	}
	// Removes an entity from the system
	pub fn remove_entity(&mut self, entity_id: u16) -> Box<Entity> {
		self.entities.remove(entity_id as usize)
	}
}

// A simple entity in the world
#[derive(Clone)]
pub struct Entity {
	pub name: String,
	pub entity_id: u16,
	pub components_bitfield: u8,
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
		self.components_bitfield = self.components_bitfield | component_id;
		self.components.insert(component_id, world_component_id as u16);
		println!("Link component '{}' to entity '{}', with ID {}", component_name, self.name, component_id);
	}
	// Unlink a component from this entity
	pub fn unlink_component<T: ComponentID>(&mut self, world: &mut World) {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(&name);
		// Take the bit, invert it, then AND it to the bitfield
		self.components_bitfield = (!id) & self.components_bitfield;
		self.components.remove(&id);
	}
	// Gets a specific component
	pub fn get_component<'a, T: ComponentID, U: Component>(&'a self, world: &'a mut World) -> &Box<dyn Component> {
		let name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&name);
		let entity_component_id = self.components[&component_id];
		let final_component = &world.component_manager.components[entity_component_id as usize];
		final_component
	}
}

// Default
impl Default for Entity {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed Entity"),
			entity_id: 0,
			components_bitfield: 0,
			components: HashMap::new(),
		}
	}
}