use std::{any::{Any}, collections::HashMap, hash::Hash};
use super::error::ECSError;


// A component trait that can be added to other components
pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    component_ids: HashMap<String, u16>,
    pub linked_entity_components: HashMap<u16, LinkedComponents>,
    pub current_component_id: u16,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            component_ids: HashMap::new(),
            linked_entity_components: HashMap::new(),
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
        println!("Register component {} with ID {}", name, component_id);
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
            println!("GetComponentID call '{}' {}", name, value);
            Ok(value)
        } else {
            return Err(ECSError::new(format!("Component {} not registered!", name).as_str()));
        }
    }
    // Checks if a specific component is registered
    pub fn is_component_registered<T: ComponentID>(&self) -> bool {
        self.component_ids.contains_key(&T::get_component_name())
    }
    // Add a specific LinkedComponents struct to the world
    pub fn add_linkedentitycomponents(&mut self, entity_id: u16, lec: LinkedComponents) -> Result<&mut LinkedComponents, ECSError> {
        self.linked_entity_components.insert(entity_id, lec);
        // Give back a mutable reference
        return Ok(self.get_linkedentitycomponents_mut(entity_id)?);
    }
    // Get a reference to a specific linked entity components struct
    pub fn get_linkedentitycomponents(&self, entity_id: u16) -> Result<&LinkedComponents, ECSError> {
        let linked_entity_components = self.linked_entity_components.get(&entity_id).unwrap();
        return Ok(linked_entity_components);
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn get_linkedentitycomponents_mut(&mut self, entity_id: u16) -> Result<&mut LinkedComponents, ECSError> {
        let linked_entity_components = self.linked_entity_components.get_mut(&entity_id).unwrap();
        return Ok(linked_entity_components);
    }
    // Remove a specified linked entity component struct from the manager
    pub fn remove_linkedentitycomponents(&mut self, entity_id: &u16) -> Result<(), ECSError> {
        self.linked_entity_components.remove(entity_id).unwrap();
        return Ok(());
    }
}

// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}

// Components that are linked to a specific entity
#[derive(Default)]
pub struct LinkedComponents {
	pub components: HashMap<u16, Box<dyn Component>>,
}

// Get a specific component from the linked entity components struct
impl LinkedComponents {
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
	// Get a mutable reference to a specific component
	pub fn get_component_mut<'a, T: Component + ComponentID + 'static>(&'a mut self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
		let component_id = component_manager.get_component_id::<T>()?;
		let component = Self::cast_component_mut(self.components.get_mut(&component_id).unwrap());
		Ok(component)
	}
    // Get a reference to a specific component using it's component ID
    pub fn id_get_component<'a, T: Component + ComponentID + 'static>(&'a self, component_id: &u16) -> Result<&'a T, ECSError> {
		let component = Self::cast_component(self.components.get(&component_id).unwrap());
		Ok(component)
	} 
    // Get a mutable reference to a specific component using it's component ID
    pub fn id_get_component_mut<'a, T: Component + ComponentID + 'static>(&'a mut self, component_id: &u16) -> Result<&'a mut T, ECSError> {
		let component = Self::cast_component_mut(self.components.get_mut(&component_id).unwrap());
		Ok(component)
	} 
    // Add a single component to the LinkedComponents
    pub fn id_add_component<'a, T: ComponentID + Component + 'static>(
        &mut self,
        component: T,
        component_id: u16
    ) {
        // Box the component
        let boxed_component = Box::new(component);
        self.components.insert(component_id, boxed_component);
    }
    // Remove a single component from the component manager using it's id
    pub fn remove_component(&mut self, component_id: &u16) {
        self.components.remove(component_id);
    }
    // Check if this linked entity components contains a specific component
    pub fn contains_component(&self, component_id: &u16) -> bool { self.components.contains_key(component_id) }
}

// The filtered components that are linked to a specific entity, and that also match a specific c_bitfield
pub struct FilteredLinkedComponents {
    pub entity_id: u16
}

// Get the components
impl FilteredLinkedComponents {
    // Get a reference to a component using the component manager
    pub fn get_component<'a, T: Component + ComponentID + 'static>(&'a self, component_manager: &'a ComponentManager) -> Result<&'a T, ECSError> {
        let id = component_manager.get_component_id::<T>()?.clone();        
        let lec = component_manager.get_linkedentitycomponents(self.entity_id)?;
        let component = lec.id_get_component::<T>(&id)?;
        return Ok(component);
    }
    // Get a mutable reference to a component using the component manager
    pub fn get_component_mut<'a, T: Component + ComponentID + 'static>(&'a mut self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
        let id = component_manager.get_component_id::<T>()?.clone();
        println!("Get filtered component call");
        let lec = component_manager.get_linkedentitycomponents_mut(self.entity_id)?;
        let component = lec.id_get_component_mut::<T>(&id)?;
        return Ok(component);
    }
}