use std::collections::HashMap;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u8 = 32;

// A component trait that can be added to other components
pub trait Component {
	fn get_component_id() -> u8 {
		0
	}
	fn get_component_name() -> String {
		String::from("Une patate")
	}
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentID {
	components: HashMap<String, i8>,	
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
	fn get_component_id<Component>(&mut self) -> u8 {
		0
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
	pub components_id: u8,
}

// Default
impl Default for Entity {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed Entity"),
			components_id: 0,
		}
	}
}