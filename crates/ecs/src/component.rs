use crate::{
    ECSError,
};

use others::SmartList;
use std::any::Any;

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentManager {
    pub components: SmartList<Box<dyn ComponentInternal + Sync + Send>>,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self { components: SmartList::default() }
    }
}

// Implement all the functions
impl ComponentManager {
    // Add a specific linked componment to the component manager, returns the global IDs of the components
    pub fn add_component(&mut self, component: Box<dyn ComponentInternal + Send + Sync + 'static>) -> Result<usize, ECSError> {
        let global_id = self.components.add_element(component);
        Ok(global_id)
    }
    // Cast a boxed component to a reference of that component
    fn cast_component<'a, T: ComponentInternal + 'static>(linked_component: &'a dyn ComponentInternal, global_id: usize) -> Result<&T, ECSError> {
        let component_any: &dyn Any = linked_component.as_any();
        let reference = component_any.downcast_ref::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))?;
        Ok(reference)
    }
    // Cast a boxed component to a mutable reference of that component
    fn cast_component_mut<'a, T: ComponentInternal + 'static>(boxed_component: &'a mut dyn ComponentInternal, global_id: usize) -> Result<&mut T, ECSError> {
        let component_any: &mut dyn Any = boxed_component.as_any_mut();
        let reference_mut = component_any.downcast_mut::<T>().ok_or_else(|| ECSError::new_str("Could not cast component"))?;
        Ok(reference_mut)
    }
    // Get a reference to a specific linked component
    pub fn get_component<'a, T: Component + 'static>(&'a self, global_id: usize) -> Result<&T, ECSError> {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let linked_component = self
            .components
            .get_element(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component::<T>(linked_component.as_ref(), global_id)?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn get_component_mut<'a, T: Component + 'static>(&'a mut self, global_id: usize) -> Result<&mut T, ECSError> {
        let linked_component = self
            .components
            .get_element_mut(global_id)
            .unwrap()
            .ok_or_else(|| ECSError::new(format!("Linked component with global ID: '{}' could not be fetched!", global_id)))?;
        let component = Self::cast_component_mut::<T>(linked_component.as_mut(), global_id)?;
        Ok(component)
    }
    // Remove a specified component from the list
    pub fn remove_component(&mut self, global_id: usize) -> Result<(), ECSError> {
        // To remove a specific component just set it's component slot to None
        self.components.remove_element(global_id).unwrap();
        Ok(())
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
pub trait ComponentID
where
    Self: Sized,
{
    fn get_component_name() -> String;
    // Wrappers
    fn get_component_id() -> usize {
        crate::registry::get_component_id::<Self>()
    }
    fn is_registered() -> bool {
        crate::registry::is_component_registered::<Self>()
    }
}
