use std::collections::HashMap;
use std::any::type_name;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u8 = 32;

// A component trait that can be added to other components
pub trait Component {
	fn get_component_id(&mut self) -> u8 {
		
	}
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentID {
	components: HashMap,	
}

// Implement all the methods
impl ComponentID {
	pub fn new() -> Self { 
		Self {
			components = HashMap::new(),
		}
	}
	// Get the component ID of a specific component
	fn get_component_id<T>(&mut self) {
		
	}


}