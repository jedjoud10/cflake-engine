use crate::ECSError;

use super::entity::Entity;
use others::SmartList;
use std::{any::Any, collections::HashMap, sync::{atomic::AtomicUsize, Arc, RwLock}};

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    pub components: SmartList<Box<dyn ComponentInternal + Sync + Send>>,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            components: SmartList::default(),
        }
    }
}

// Implement all the functions
impl ComponentManager {    
    // Add a specific linked componment to the component manager, returns the global IDs of the components
    pub fn add_linked_component<T: Component + ComponentID + 'static>(&mut self, component: T) -> Result<usize, ECSError> {
        let global_id = self.components.add_element(Box::new(component));
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
            .components
            .get_element(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component(linked_component.as_ref())?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn id_get_linked_component_mut<T: Component + 'static>(&mut self, global_id: usize) -> Result<&mut T, ECSError> {
        let linked_component = self
            .components
            .get_element_mut(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component_mut(linked_component.as_mut())?;
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn id_remove_linked_component(&mut self, global_id: usize) -> Result<(), ECSError> {
        // To remove a specific component just set it's component slot to None
        self.components.remove_element(global_id).unwrap();
        return Ok(());
    }
}
// The main component trait
// We do a little bit of googling https://stackoverflow.com/questions/26983355/is-there-a-way-to-combine-multiple-traits-in-order-to-define-a-new-trait
pub trait Component: ComponentInternal + ComponentID + Sync + Send {}
// A component trait that can be added to other components
pub trait ComponentInternal {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
// A trait used to identify each component by their name
pub trait ComponentID {
    fn get_component_name() -> String;
}
