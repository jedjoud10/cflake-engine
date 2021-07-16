use std::collections::HashMap;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u8 = 32;

// A component trait that can be added to other components
pub trait Component {
	fn get_component_name() -> String {
		String::from("Une patate")
	}
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentID {
	pub components: HashMap<String, u8>,	
}

// Implement default values
impl Default for ComponentID {
	fn default() -> Self { 
		Self {
			components: HashMap::new(),
		}
	}
}

// Implement all the functions
impl ComponentID {
	// Get the component ID of a specific component
	pub fn get_component_id<T: Component>(&mut self) -> u8 {
		let name: String = T::get_component_name();
		// It found the component, so just return it's id
		if self.components.contains_key(&name) {
			let value = self.components.get(&name).unwrap();
			value.a
		}
		
		// It did not find the component, so create a new one
		self.components.insert(name, self.components.len() as u8);
		self.components.len() as u8
	}
}

// A system that can write/read component data, every frame, or at the start of the game
pub trait System {

	// Basic control code
	fn system_addded(&mut self);
	fn system_enabled(&mut self);
	fn system_disabled(&mut self);
	fn update_system(&mut self);

	// Cannot have fields in Rust so I need to do this instead
	fn get_component_id(&mut self) -> u8;
	fn set_component_id(&mut self, id: u8);

	fn add_component(&mut self, id: u8);
}

// A simple entity in the world
pub struct Entity {
	pub name: String,
	pub entity_id: usize,
	pub components_id: u8,
}

// ECS time bois
impl Entity {
	pub fn add_component(&mut self, id: u8) {

	}
	pub fn remove_component(&mut self, id: u8) {
		
	}
}

// Default
impl Default for Entity {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed Entity"),
			entity_id: 0,
			components_id: 0,
		}
	}
}