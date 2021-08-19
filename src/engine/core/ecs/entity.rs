
use std::{collections::HashMap};

use super::component::{Component, ComponentID, ComponentManager};
use super::error::ECSError;
// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub c_bitfield: u16,
    // The actual components are stored in the world
    pub components: HashMap<u16, u16>,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default_component<T: ComponentID + Default + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
    ) -> Result<(), ECSError> {
        // Simple wrapper around the default link component
        self.link_component(component_manager, T::default())
    }
    // Link a component to this entity and use the given default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
        default_state: T,
    ) -> Result<(), ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we have the component linked on this entity
        if self.components.contains_key(&component_id) {
            return Err(ECSError::new(
                format!(
                    "Cannot link component '{}' to entity '{}' because it is already linked!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }

        // Add the component inside the component manager
        let global_id = component_manager.add_component::<T>(Box::new(default_state));

        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        self.components.insert(component_id, global_id as u16);
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) {
        let _name = T::get_component_name();
        let id = component_manager.get_component_id::<T>().unwrap();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;
        let _global_id = self.components.remove(&id).unwrap();
        component_manager.remove_component(id);
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let final_component = component_manager
                .id_get_component::<T>(self.components[&component_id])
                .unwrap();
            Ok(final_component)
        } else {
            return Err(ECSError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
    // Gets a specific component, mutably
    pub fn get_component_mut<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a mut ComponentManager,
    ) -> Result<&'a mut T, ECSError> {
        let component_id = component_manager.get_component_id::<T>()?;
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let final_component = component_manager
                .id_get_component_mut::<T>(self.components[&component_id])
                .unwrap();
            Ok(final_component)
        } else {
            return Err(ECSError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
    // Get the global world ID of a specified component that this entity has
    pub fn get_global_component_id<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a mut ComponentManager,
    ) -> Result<u16, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            Ok(self.components[&component_id])
        } else {
            return Err(ECSError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
	// Get all the components (local ID hashmap) that match with the specified bitfield
	pub fn bitfield_get_components(&self, bitfield: u16) -> HashMap<u16, u16> {
		// Loop over all the components and filter them
		let components = self.components.iter().filter(|(&component_id, _)| {
			// Create a bitwise AND with the bitfield and component ID...
			// Then check if it is equal to the component ID
			(bitfield & component_id) == component_id
		}).map(|x| (*x.0, *x.1)).collect();
		components
	}
}
