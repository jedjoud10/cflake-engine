use std::error::Error;
use std::{any::Any, collections::HashMap};

use super::component::{Component, ComponentID, ComponentManager};
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
    ) -> Result<(), super::error::EntityError> {
        // Simple wrapper around the default link component
        self.link_component(component_manager, T::default())
    }
    // Link a component to this entity and use the given default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
        default_state: T,
    ) -> Result<(), super::error::EntityError> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we have the component linked on this entity
        if self.components.contains_key(&component_id) {
            return Err(super::error::EntityError::new(
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
        self.c_bitfield = self.c_bitfield | component_id;
        self.components.insert(component_id, global_id as u16);
        println!(
            "Link component '{}' to entity '{}', with ID: {} and global ID: '{}'",
            T::get_component_name(),
            self.name,
            component_id,
            global_id
        );
        return Ok(());
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) {
        let name = T::get_component_name();
        let id = component_manager.get_component_id::<T>();
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield = (!id) & self.c_bitfield;
        let global_id = self.components.remove(&id).unwrap();
        component_manager.remove_component(id);
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, super::error::EntityError> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let final_component = component_manager
                .id_get_component::<T>(self.components[&component_id])
                .unwrap();
            return Ok(final_component);
        } else {
            return Err(super::error::EntityError::new(
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
    ) -> Result<&'a mut T, super::error::EntityError> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let final_component = component_manager
                .id_get_component_mut::<T>(self.components[&component_id])
                .unwrap();
            return Ok(final_component);
        } else {
            return Err(super::error::EntityError::new(
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
    ) -> Result<u16, super::error::EntityError> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            return Ok(self.components[&component_id]);
        } else {
            return Err(super::error::EntityError::new(
                format!(
                    "Component '{}' does not exist on Entity '{}'!",
                    T::get_component_name(),
                    self.name
                )
                .as_str(),
            ));
        }
    }
}
