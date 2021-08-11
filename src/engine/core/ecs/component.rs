use std::{any::Any, collections::HashMap};

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u16 = 16;

// A component trait that can be added to other components
pub trait Component {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
	pub component_ids: HashMap<String, u16>,
	pub components: Vec<Box<dyn Component>>,
	pub discrete_components: Vec<Box<dyn Component>>,
	pub current_component_id: u16,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
			component_ids: HashMap::new(),
			components: Vec::new(),
			discrete_components: Vec::new(),
			current_component_id: 1,
		}
    }
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
}

// A trait used to identify each component by their name
pub trait ComponentID {
	fn get_component_name() -> String;
}
