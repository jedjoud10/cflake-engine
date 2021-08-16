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
    component_ids: HashMap<String, u16>,
    components: HashMap<u16, Box<dyn Component>>,
    pub current_component_id: u16,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            component_ids: HashMap::new(),
            components: HashMap::new(),
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
    pub fn name_get_component_id(&self, name: &String) -> u16 {
        // It found the component, so just return it's id
        if self.component_ids.contains_key(name) {
            let value = self.component_ids[name];
            return value;
        } else {
            panic!("Component {} not registered!", name);
        }
    }
	// Cast a boxed component to a reference of that component
	pub fn cast_component<'a, T: ComponentID + Component + 'static>(boxed_component: &'a Box<dyn Component>) -> &'a T {
		let component_any: &dyn Any = boxed_component.as_any();
        let final_component = component_any.downcast_ref::<T>().unwrap();
		final_component
	}
	
	// Cast a boxed component to a mutable reference of that component
	pub fn cast_component_mut<'a, T: ComponentID + Component + 'static>(boxed_component: &'a mut Box<dyn Component>) -> &'a mut T {
		let component_any: &mut dyn Any = boxed_component.as_any_mut();
        let final_component = component_any.downcast_mut::<T>().unwrap();
		final_component
	}
	// Check if we have a specified component in the manager
	pub fn is_component_id_valid(&self, component_id: u16) -> bool {
		self.components.contains_key(&component_id)
	}
    // Get a refernece to a component by it's global ID
    pub fn id_get_component<'a, T: ComponentID + Component + 'static>(
        &'a self,
        id: u16,
    ) -> Result<&'a T, super::error::ComponentError> {
        // Check if we even have the component
        if (id as usize) < self.components.len() {
            return Ok(Self::cast_component::<T>(self.components.get(&id).unwrap()));
        } else {
            return Err(super::error::ComponentError::new(
                format!(
                    "Component '{}' does not exist in the ComponentManager!",
                    T::get_component_name()
                )
                .as_str(),
            ));
        }
    }
	// Get a mutable component by it's global ID
    pub fn id_get_component_mut<'a, T: ComponentID + Component + 'static>(
        &'a mut self,
        id: u16,
    ) -> Result<&'a mut T, super::error::ComponentError> {
        // Check if we even have the component
        if (id as usize) < self.components.len() {
            return Ok(Self::cast_component_mut::<T>(self.components.get_mut(&id).unwrap()));
        } else {
            return Err(super::error::ComponentError::new(
                format!(
                    "Component '{}' does not exist in the ComponentManager!",
                    T::get_component_name()
                )
                .as_str(),
            ));
        }
    }
	// Add a single component to the component manager
	pub fn add_component<'a, T: ComponentID + Component + 'a>(&mut self, component: Box<dyn Component>) -> u16 {
		let id = self.components.len() as u16;
		self.components.insert(id,component);
		id
	}
	// Remove a single component from the component manager using it's id
	pub fn remove_component(&mut self, component_id: u16) {
		self.components.remove(&component_id);
	}
}

// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}
