use super::{entity::Entity, error::ECSError};
use std::{any::Any, collections::HashMap};

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    component_ids: HashMap<String, u16>,
    pub linked_component: HashMap<u16, Box<dyn ComponentInternal + Send + Sync>>,
    pub current_component_id: u16,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            component_ids: HashMap::new(),
            linked_component: HashMap::new(),
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
        self.component_ids.insert(name, self.current_component_id);
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
    // Add a specific linked componment to the component manager, returns the global IDs of the components
    pub fn add_linked_component<T: Component + ComponentID + Send + Sync + 'static>(&mut self, component: T) -> Result<u16, ECSError> {
        let global_id = self.linked_component.len() as u16;
        let boxed_component = Box::new(component);
        self.linked_component.insert(global_id, boxed_component);
        // Give back the global ID of the component
        Ok(global_id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T: ComponentInternal + 'static>(boxed_component: &'a Box<dyn ComponentInternal + Send + Sync>) -> &'a T {
        let component_any: &dyn Any = boxed_component.as_any();
        let final_component = component_any.downcast_ref::<T>().unwrap();
        final_component
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T: ComponentInternal + 'static>(boxed_component: &'a mut Box<dyn ComponentInternal + Send + Sync>) -> &'a mut T {
        let component_any: &mut dyn Any = boxed_component.as_any_mut();
        let final_component = component_any.downcast_mut::<T>().unwrap();
        final_component
    }
    // Get a reference to a specific linked component
    pub fn id_get_linked_component<T: Component + 'static>(&self, global_id: &u16) -> Result<&T, ECSError> {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let linked_component = self.linked_component.get(global_id).unwrap();
        let component = Self::cast_component(linked_component);
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn id_get_linked_component_mut<T: Component + 'static>(&mut self, global_id: &u16) -> Result<&mut T, ECSError> {
        let linked_component = self.linked_component.get_mut(global_id).unwrap();
        let component = Self::cast_component_mut(linked_component);
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn id_remove_linked_component(&mut self, _global_id: &u16) -> Result<(), ECSError> {
        //self.linked_entity_components.remove(entity_id).unwrap();
        todo!();
    }
}
// The main component trait
// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
pub trait Component: ComponentInternal + ComponentID + Send + Sync + Default {}
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
pub struct FilteredLinkedComponents {
    pub entity_id: u16,
    pub components: HashMap<u16, u16>,
}

// Get the components
impl FilteredLinkedComponents {
    // Get the matching filtered components from a specific entity
    pub fn get_filtered_linked_components(entity: &Entity, system_c_bitfield: u16) -> Self {
        let mut filtered_linked_components: Self = Self::default();
        let global_ids: HashMap<u16, u16> = entity
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
        filtered_linked_components.entity_id = entity.entity_id;
        filtered_linked_components
    }
    // Get a reference to a component using the component manager
    pub fn get_component<'a, T: Component + ComponentID + Sync + 'static>(&'a self, component_manager: &'a ComponentManager) -> Result<&'a T, ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        // Check if we are even allowed to get that components
        if self.components.contains_key(&id) {
            // We are allowed to get this component
            let global_id = self.components.get(&id).unwrap();
            let component = component_manager.id_get_linked_component(global_id)?;
            Ok(component)
        } else {
            // We are not allowed to get this component
            return Err(ECSError::new(
                format!("Cannot get component with ID: '{}' from FilteredLinkedComponents for entity ID: {}", id, self.entity_id).as_str(),
            ));
        }
    }
    // Get a mutable reference to a component using the component manager
    pub fn get_component_mut<'a, T: Component + ComponentID + Sync + 'static>(&'a self, component_manager: &'a mut ComponentManager) -> Result<&'a mut T, ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        // Check if we are even allowed to get that components
        if self.components.contains_key(&id) {
            // We are allowed to get this component
            let global_id = self.components.get(&id).unwrap();
            let component = component_manager.id_get_linked_component_mut(global_id)?;
            Ok(component)
        } else {
            // We are not allowed to get this component
            return Err(ECSError::new(
                format!("Cannot get component with ID: '{}' from FilteredLinkedComponents for entity ID: {}", id, self.entity_id).as_str(),
            ));
        }
    }
}
