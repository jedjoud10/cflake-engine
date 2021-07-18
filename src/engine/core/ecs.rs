use std::collections::HashMap;
use crate::engine::core::world::World;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u16 = 16;

// A component trait that can be added to other components
pub trait Component {
}

pub trait ComponentNames {
	fn get_component_name() -> String;
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentID {
	pub components: HashMap<String, u16>,	
	pub current_component_id: u16
}

// Implement default values
impl Default for ComponentID {
	fn default() -> Self { 
		Self {
			components: HashMap::new(),
			current_component_id: 1,
		}
	}
}

// Implement all the functions
impl ComponentID {
	// Get the component id for a specific entity
	pub fn get_component_id<T: ComponentNames>(&mut self) -> u16 {
		let name: String = T::get_component_name();
		
		// It found the component, so just return it's id
		if self.components.contains_key(&name) {
			let value = self.components[&name];
			return value;
		}
		// It did not find the component, so create a new "id binding" for one
		self.components.insert(name, self.current_component_id);

		// Make a copy of the id before the bit shift
		let component_id = self.current_component_id;		
		// Bit shift to the left
		self.current_component_id = self.current_component_id << 1;
		// Return the component id before the bit shift
		component_id
	}

	// Get the component id for a specific entity
	pub fn get_component_id_by_name(&mut self, name: &String) -> u16 {
		// It found the component, so just return it's id
		if self.components.contains_key(name) {
			let value = self.components[name];
			return value;
		}
		
		// It did not find the component, so create a new "id binding" for one
		let name_val = String::from(name);
		self.components.insert(name_val, self.current_component_id);

		// Make a copy of the id before the bit shift
		let component_id = self.current_component_id;		
		// Bit shift to the left
		self.current_component_id = self.current_component_id << 1;
		// Return the component id before the bit shift
		component_id
	}
}

// Tells you the state of the system, and for how long it's been enabled/disabled
pub enum SystemState {
	Enabled(f32),
	Disabled(f32)
}

// A system that can write/read component data, every frame, or at the start of the game
pub struct System {
	pub name: String,
	pub component_bitfield: u16,
	pub state: SystemState,
	pub entity_loop: fn(&mut Entity),
	pub entities: Vec<usize>
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
	// Update the system
	pub fn update_system(&mut self, world: &World, entities: &mut Vec<Box<Entity>>) {
		// Loop over all the entities and update their components
		for entity in self.entities.iter() {		
			(self.entity_loop)(entities.get_mut(*entity).unwrap());
		}
	}
	// Add a component to this system's component bitfield id
	pub fn link_component<T: ComponentNames>(&mut self, world: &mut World) {
		self.component_bitfield = self.component_bitfield | world.component_manager.get_component_id::<T>();
	}
	// Adds an entity to the system
	pub fn add_entity(&mut self, entity: &Entity) {
		println!("Added entity '{}', with ID {} to the system '{}'", entity.name, entity.entity_id, self.name);
		self.entities.push(entity.entity_id);
	}
}

impl System {
	pub fn new(name: String) -> Self {
		let empty_entity_loop = |_entity: &mut Entity| {};
		Self {
			name,
			component_bitfield: 0,
			state: SystemState::Disabled(0.0),
			entity_loop: empty_entity_loop,
			entities: Vec::new()
		}
	}
}

// A simple entity in the world
pub struct Entity {
	pub name: String,
	pub entity_id: usize,
	pub components_bitfield: u16,
	components: HashMap<u16, Box<dyn Component>>,
}

// ECS time bois
impl Entity {
	// Link a component to this entity
	pub fn link_component<T: ComponentNames, U: Component + 'static>(&mut self, world: &mut World, component: U) {
		let component_name = T::get_component_name();
		let component_id = world.component_manager.get_component_id_by_name(&component_name);
		self.components_bitfield = self.components_bitfield | component_id;
		self.components.insert(component_id, Box::new(component));
		println!("Link component '{}' to entity '{}', with ID {}", component_name, self.name, component_id);
	}
	// Unlink a component from this entity
	pub fn unlink_component<T: ComponentNames>(&mut self, world: &mut World) {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(&name);
		// Take the bit, invert it, then AND it to the bitfield
		self.components_bitfield = (!id) & self.components_bitfield;
		self.components.remove(&id);
	}
	// Gets a specific component
	pub fn get_component<T: ComponentNames, U: Component>(&mut self, world: &mut World) -> &Box<dyn Component> {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(&name);
		let value = &self.components[&id];
		value
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