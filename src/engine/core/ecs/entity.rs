use std::error::Error;
use std::{any::Any, collections::HashMap};

use super::component::{Component, ComponentID, ComponentManager};
// A simple entity in the world
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_id: u16,
    pub c_bitfield: u16,
    // The actual components are stored in the world, this allows for two objects to share a single component if we want to have duplicate entities
    components: HashMap<u16, u16>,
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
    ) {
        let component_name = T::get_component_name();
        let component_id = component_manager.get_component_id_by_name(&component_name);
        // Check if we have the component linked in the first place
        if self.components.contains_key(&component_id) {
            println!(
                "Cannot link component '{}' to entity '{}' because it is already linked!",
                component_name, self.name
            );
            return;
        }
        component_manager.components.push(Box::new(T::default()));
        let world_component_id = component_manager.components.len() - 1;
        self.c_bitfield = self.c_bitfield | component_id;
        self.components
            .insert(component_id, world_component_id as u16);
        println!(
            "Link component '{}' to entity '{}', with ID: {} and global ID: '{}'",
            component_name, self.name, component_id, world_component_id
        );
    }
    // Link a component to this entity and use the gived default state parameter
    pub fn link_component<T: ComponentID + Component + 'static>(
        &mut self,
        component_manager: &mut ComponentManager,
        default_state: T,
    ) {
        let component_name = T::get_component_name();
        let component_id = component_manager.get_component_id_by_name(&component_name);
        // Check if we have the component linked in the first place
        if self.components.contains_key(&component_id) {
            println!(
                "Cannot link component '{}' to entity '{}' because it is already linked!",
                component_name, self.name
            );
            return;
        }
        component_manager.components.push(Box::new(default_state));
        let world_component_id = component_manager.components.len() - 1;
        self.c_bitfield = self.c_bitfield | component_id;
        self.components
            .insert(component_id, world_component_id as u16);
        println!(
            "Link component '{}' to entity '{}', with ID: {} and global ID: '{}'",
            component_name, self.name, component_id, world_component_id
        );
    }
    // Unlink a component from this entity
    pub fn unlink_component<T: ComponentID>(&mut self, component_manager: &ComponentManager) {
        let name = T::get_component_name();
        let id = component_manager.get_component_id_by_name(&name);
        // Take the bit, invert it, then AND it to the bitfield
        self.c_bitfield = (!id) & self.c_bitfield;
        self.components.remove(&id);
    }
    // Gets a reference to a component
    pub fn get_component<'a, T: ComponentID + Component + 'static>(
        &self,
        component_manager: &'a ComponentManager,
    ) -> Result<&'a T, super::error::Error> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let component_any: &dyn Any = component_manager
                .components
                .get(self.components[&component_id] as usize)
                .unwrap()
                .as_any();
            let final_component = component_any.downcast_ref::<T>().unwrap();
            return Ok(final_component);
        } else {
            return Err(super::error::Error::new(
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
    ) -> Result<&'a mut T, super::error::Error> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            let component_any: &mut dyn Any = component_manager
                .components
                .get_mut(self.components[&component_id] as usize)
                .unwrap()
                .as_any_mut();
            let final_component = component_any.downcast_mut::<T>().unwrap();
            return Ok(final_component);
        } else {
            return Err(super::error::Error::new(
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
    ) -> Result<u16, super::error::Error> {
        let component_id = component_manager.get_component_id::<T>();
        // Check if we even have the component
        if self.components.contains_key(&component_id) {
            return Ok(self.components[&component_id]);
        } else {
            return Err(super::error::Error::new(
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
