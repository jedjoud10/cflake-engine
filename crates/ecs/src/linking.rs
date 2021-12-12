use crate::{Component, ComponentID, ComponentInternal, ECSError, Entity};
use std::{collections::HashMap, fmt::Debug};

// A collection of components that will be mass linked to a specific entity when it gets added into the world on the main thread
pub struct ComponentLinkingGroup {
    pub linked_components: HashMap<usize, Box<dyn ComponentInternal + Sync + Send>>,
    pub c_bitfield: usize,
}

impl Debug for ComponentLinkingGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentLinkingGroup").field("c_bitfield", &self.c_bitfield).finish()
    }
}

// Linking methods
impl ComponentLinkingGroup {
    pub fn empty() -> Self {
        Self {
            linked_components: HashMap::new(),
            c_bitfield: 0,
        }
    }
    // Creete a new component linking group from an entity
    pub fn new(entity: &Entity) -> Self {
        Self {
            linked_components: HashMap::new(),
            c_bitfield: entity.c_bitfield,
        }
    }
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default<T: Component + Default + 'static>(&mut self) -> Result<(), ECSError> {
        // Simple wrapper around the default link component
        self.link(T::default())
    }
    // Check if we have a component linked
    pub fn is_component_linked(&self, component_id: usize) -> bool {
        self.linked_components.contains_key(&component_id)
    }
    // Link a component to this entity and also link it's default component dependencies if they are not linked yet
    pub fn link<T: Component + 'static>(&mut self, default_state: T) -> Result<(), ECSError> {
        let component_id = crate::registry::get_component_id::<T>()?;
        // Check if we have the component linked on this entity
        if let std::collections::hash_map::Entry::Vacant(e) = self.linked_components.entry(component_id) {
            // Add the local component to our hashmap
            let boxed = Box::new(default_state);
            e.insert(boxed);
        } else {
            // The component was already linked
            return Err(ECSError::new(format!(
                "Cannot link component '{}' to ComponentLinkingGroup because it is already linked!",
                T::get_component_name(),
            )));
        }
        // Add the component's bitfield to the entity's bitfield
        self.c_bitfield |= component_id;
        Ok(())
    }
}
