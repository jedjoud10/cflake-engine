use crate::ECSError;

use super::entity::Entity;
use others::SmartList;
use std::{any::Any, collections::HashMap};

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    component_ids: HashMap<String, usize>,
    pub smart_components_list: SmartList<Box<dyn ComponentInternal>>,
    pub current_component_id: usize,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            component_ids: HashMap::new(),
            smart_components_list: SmartList::default(),
            current_component_id: 1,
        }
    }
}

// Implement all the functions
impl ComponentManager {
    // Registers a specific component
    pub fn register_component<T: ComponentID>(&mut self) -> usize {
        let name: String = T::get_component_name();
        // Register the component
        self.component_ids.insert(name, self.current_component_id);
        // Make a copy of the id before the bit shift
        let component_id = self.current_component_id;
        // Bit shift to the left
        self.current_component_id <<= 1;
        // Return the component id before the bit shift
        component_id
    }
    // Get the component id for a specific entity
    pub fn get_component_id<T: ComponentID>(&self) -> Result<usize, ECSError> {
        let name: String = T::get_component_name();
        // It found the component, so just return it's id
        if self.component_ids.contains_key(&name) {
            let value = self.component_ids[&name];
            Ok(value)
        } else {
            return Err(ECSError::new(format!("Component {} not registered!", name)));
        }
    }
    // Checks if a specific component is registered
    pub fn is_component_registered<T: ComponentID>(&self) -> bool {
        self.component_ids.contains_key(&T::get_component_name())
    }
    // Add a specific linked componment to the component manager, returns the global IDs of the components
    pub fn add_linked_component<T: Component + ComponentID + 'static>(&mut self, component: T) -> Result<usize, ECSError> {
        let global_id = self.smart_components_list.add_element(Box::new(component));
        Ok(global_id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T: ComponentInternal + 'static>(boxed_component: &'a dyn ComponentInternal) -> Result<&'a T, ECSError> {
        let component_any: &dyn Any = boxed_component.as_any();
        component_any.downcast_ref::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T: ComponentInternal + 'static>(boxed_component: &'a mut dyn ComponentInternal) -> Result<&'a mut T, ECSError> {
        let component_any: &mut dyn Any = boxed_component.as_any_mut();
        component_any.downcast_mut::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))
    }
    // Get a reference to a specific linked component
    pub fn id_get_linked_component<T: Component + 'static>(&self, global_id: usize) -> Result<&T, ECSError> {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let linked_component = self
            .smart_components_list
            .get_element(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component(linked_component.as_ref())?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn id_get_linked_component_mut<T: Component + 'static>(&mut self, global_id: usize) -> Result<&mut T, ECSError> {
        let linked_component = self
            .smart_components_list
            .get_element_mut(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component_mut(linked_component.as_mut())?;
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn id_remove_linked_component(&mut self, global_id: usize) -> Result<(), ECSError> {
        // To remove a specific component just set it's component slot to None
        self.smart_components_list.remove_element(global_id).unwrap();
        return Ok(());
    }
}
// The main component trait
// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
pub trait Component: ComponentInternal + ComponentID {}
// A component trait that can be added to other components
pub trait ComponentInternal {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}
// The filtered components that are linked to a specific entity, and that also match a specific c_bitfield
#[derive(Default)]
pub struct LinkedComponents {
    pub components: HashMap<usize, usize>,
}

// Get the components
impl LinkedComponents {
    // Get the matching filtered components from a specific entity
    pub fn get_linked_components(entity: &Entity, system_c_bitfield: usize) -> Self {
        let mut filtered_linked_components: Self = Self::default();
        let global_ids: HashMap<usize, usize> = entity
            .linked_components
            .iter()
            .filter(|(&component_id, _)| {
                // Create a bitwise AND with the bitfield and component ID...
                // Then check if it is equal to the component ID
                (system_c_bitfield & component_id) == component_id
            })
            .map(|(&x, &x1)| (x, x1))
            .collect();
        filtered_linked_components.components = global_ids;
        filtered_linked_components
    }
    // Get a reference to a component using the component manager
    pub fn get_component<'a, T: Component + ComponentID + 'static>(&'a self, component_manager: &'a ComponentManager) -> Result<&'a T, ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        // Check if we are even allowed to get that components
        if self.components.contains_key(&id) {
            // We are allowed to get this component
            let global_id = self.components.get(&id).unwrap();
            let component = component_manager.id_get_linked_component(*global_id)?;
            Ok(component)
        } else {
            // We are not allowed to get this component
            return Err(ECSError::new(format!("Cannot get component with ID: '{}' from FilteredLinkedComponents!", id)));
        }
    }
    // Get a mutable reference to a component using the component manager
    pub fn get_component_mut<'a, T: Component + ComponentID + 'static>(&'a self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        // Check if we are even allowed to get that components
        if self.components.contains_key(&id) {
            // We are allowed to get this component
            let global_id = self.components.get(&id).unwrap();
            let component = component_manager.id_get_linked_component_mut(*global_id)?;
            Ok(component)
        } else {
            // We are not allowed to get this component
            return Err(ECSError::new(format!("Cannot get component with ID: '{}' from FilteredLinkedComponents!", id)));
        }
    }
}
