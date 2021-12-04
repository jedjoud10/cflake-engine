use std::collections::HashMap;

use others::SmartList;

use crate::{ComponentInternal, Component, ECSError};

// A collection of components that will be mass linked to a specific entity when it gets added into the world on the main thread
pub struct ComponentLinkingGroup {
    // Components
    pub smart_components_list: HashMap<usize, Box<dyn ComponentInternal + Sync + Send>>,
}

// Linking methods
impl ComponentLinkingGroup {
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default_component<T: Component + Default + 'static>(&mut self) -> Result<(), ECSError> {
        // Simple wrapper around the default link component
        self.link_component(T::default())
    }
    // Check if we have a component linked
    pub fn is_component_linked(&self, component_id: usize) -> bool {
        self.linked_components.contains_key(&component_id)
    }
    // Link a component to this entity and also link it's default component dependencies if they are not linked yet
    pub fn link_component<T: Component + 'static>(&mut self, default_state: T) -> Result<(), ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we have the component linked on this entity
        if let std::collections::hash_map::Entry::Vacant(e) = self.linked_components.entry(component_id) {
            // The component was not linked yet, link it
            // Add the component and get the global ID and add it to our hashmap
            let global_id = component_manager.add_linked_component::<T>(default_state)?;
            // Add the global ID to our hashmap
            e.insert(global_id);
        } else {
            // The component was already linked
            return Err(ECSError::new(format!(
                "Cannot link component '{}' to entity '{}' because it is already linked!",
                T::get_component_name(),
                self.name
            )));
        }
        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        let id = component_manager.get_component_id::<T>()?;
        let global_id = *self.linked_components.get(&id).unwrap();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;

        // Get the linked components and remove the component from it
        component_manager.id_remove_linked_component(global_id)?;
        Ok(())
    }
}