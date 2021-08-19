use std::{any::{Any}, collections::HashMap};
use super::error::ECSError;


// A component trait that can be added to other components
pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    component_ids: HashMap<String, u16>,
    pub linked_entity_components: Vec<LinkedEntityComponents>,
    pub current_component_id: u16,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            component_ids: HashMap::new(),
            linked_entity_components: Vec::new(),
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
            .insert(name, self.current_component_id);
        // Make a copy of the id before the bit shift
        let component_id = self.current_component_id;
        // Bit shift to the left
        self.current_component_id <<= 1;
        // Return the component id before the bit shift
        component_id
    }
    // Get the component id for a specific entity
    pub fn get_component_id<T: ComponentID>(&self) -> Result<u16, ECSError> {
        let name: String = T::get_component_name();
        // It found the component, so just return it's id
        if self.component_ids.contains_key(&name) {
            let value = self.component_ids[&name];
            Ok(value)
        } else {
            return Err(ECSError::new(format!("Component {} not registered!", name).as_str()));
        }
    }
    // Checks if a specific component is registered
    pub fn is_component_registered<T: ComponentID>(&self) -> bool {
        self.component_ids.contains_key(&T::get_component_name())
    }
}

// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}

// Components that are linked to a specific entity
pub struct LinkedEntityComponents {
	pub components: HashMap<u16, Box<dyn Component>>,
}

// Get a specific component from the linked entity components struct
impl LinkedEntityComponents {
    // Cast a boxed component to a reference of that component
    pub fn cast_component<'a, T: ComponentID + Component + 'static>(
        boxed_component: &'a Box<dyn Component>,
    ) -> &'a T {
        let component_any: &dyn Any = boxed_component.as_any();
        let final_component = component_any.downcast_ref::<T>().unwrap();
        final_component
    }
    // Cast a boxed component to a mutable reference of that component
    pub fn cast_component_mut<'a, T: ComponentID + Component + 'static>(
        boxed_component: &'a mut Box<dyn Component>,
    ) -> &'a mut T {
        let component_any: &mut dyn Any = boxed_component.as_any_mut();
        let final_component = component_any.downcast_mut::<T>().unwrap();
        final_component
    }
	// Get a reference to a specific component
	pub fn get_component<'a, T: Component + ComponentID + 'static>(&'a self, component_manager: &'a ComponentManager) -> Result<&'a T, ECSError> {
		let component_id = component_manager.get_component_id::<T>()?;
		let component = Self::cast_component(self.components.get(&component_id).unwrap());
		Ok(component)
	}
	// Get a reference to a specific component mutably
	pub fn get_component_mut<'a, T: Component + ComponentID + 'static>(&'a self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
		let component_id = component_manager.get_component_id::<T>()?;
		let component = Self::cast_component_mut(self.components.get_mut(&component_id).unwrap());
		Ok(component)
	}
}