
use std::{collections::HashMap};

use gl::LineWidth;

use super::component::{Component, ComponentID, ComponentManager, LinkedComponents};
use super::error::ECSError;
// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub lc_id: u16,
    pub generate_lc_id: bool,
    pub c_bitfield: u16,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            generate_lc_id: true,
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

        // Get the linked entity component from the component manager
        let linked_entity_component: &mut LinkedComponents;

        // Generate a new LinkedComponent ID
        if self.generate_lc_id {
            self.lc_id = component_manager.linked_entity_components.len() as u16;
            self.generate_lc_id = false;
        }

        if component_manager.linked_entity_components.contains_key(&self.lc_id) {
            // It already exists, so just use that
            linked_entity_component = component_manager.get_linkedentitycomponents_mut(self.lc_id)?;
        } else {
            linked_entity_component = component_manager.add_linkedentitycomponents(self.lc_id, LinkedComponents::default())?;
        }
        println!("{:?}", linked_entity_component.components.keys());
        // Check if we have the component linked on this entity
        if linked_entity_component.components.contains_key(&component_id) {
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
        linked_entity_component.id_add_component::<T>(default_state, component_id);

        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) {
        let _name = T::get_component_name();
        let id = component_manager.get_component_id::<T>().unwrap();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield &= !id;
        component_manager.remove_linkedentitycomponents(&self.entity_id).unwrap();
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, ECSError> {
        let component_id = component_manager.get_component_id::<T>().unwrap();        
        let lec = component_manager.get_linkedentitycomponents(self.entity_id)?;
        println!("{:?}", lec.components.keys());
        // Check if we even have the component
        if lec.contains_component(&component_id) {
            let final_component = lec.id_get_component::<T>(&component_id)?;
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
        let component_id = component_manager.get_component_id::<T>().unwrap();
        let lec = component_manager.get_linkedentitycomponents_mut(self.entity_id)?;
        println!("{:?} {}", lec.components.keys(), component_id);
        // Check if we even have the component
        if lec.contains_component(&component_id) {
            let final_component = lec.id_get_component_mut::<T>(&component_id)?;
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
	// Get all the components (local ID hashmap) that match with the specified bitfield
	pub fn bitfield_get_components(&self, bitfield: u16) -> HashMap<u16, u16> {
        todo!();
        /*
		// Loop over all the components and filter them
		let components = self.components.iter().filter(|(&component_id, _)| {
			// Create a bitwise AND with the bitfield and component ID...
			// Then check if it is equal to the component ID
			(bitfield & component_id) == component_id
		}).map(|x| (*x.0, *x.1)).collect();
		components
        */
	}
}
