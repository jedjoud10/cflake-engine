use fnv::FnvHashMap;

use crate::{bitfield::ComponentBitfield, identifiers::ComponentID, Component, ComponentLinkingError};
use std::collections::HashMap;

// A collection of components that will be mass linked to a specific entity when it gets added into the world on the main thread
pub struct ComponentLinkingGroup {
    pub linked_components: FnvHashMap<u32, Box<dyn Component + Sync + Send>>,
    pub cbitfield: ComponentBitfield,
}

// Linking methods
impl ComponentLinkingGroup {
    pub fn new() -> Self {
        Self {
            linked_components: FnvHashMap::default(),
            cbitfield: ComponentBitfield::default(),
        }
    }
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default<T: Component + Default + 'static>(&mut self) -> Result<(), ComponentLinkingError> {
        // Simple wrapper around the default link component
        self.link(T::default())
    }
    // Check if we have a component linked
    pub fn is_component_linked(&self, id: ComponentID) -> bool {
        self.linked_components.contains_key(&id.component_id)
    }
    // Link a component to this entity and also link it's default component dependencies if they are not linked yet
    pub fn link<T: Component + 'static>(&mut self, default_state: T) -> Result<(), ComponentLinkingError> {
        let cbitfield = crate::registry::get_component_bitfield::<T>();
        // Check if we have the component linked on this entity
        if let std::collections::hash_map::Entry::Vacant(e) = self.linked_components.entry(*cbitfield.bitfield) {
            // Add the local component to our hashmap
            let boxed = Box::new(default_state);
            e.insert(boxed);
        } else {
            // The component was already linked
            return Err(ComponentLinkingError::new(format!(
                "Cannot link component '{}' to ComponentLinkingGroup because it is already linked!",
                T::get_component_name(),
            )));
        }
        // Add the component's bitfield to the entity's bitfield
        self.cbitfield.bitfield = self.cbitfield.bitfield.add(&cbitfield.bitfield);
        Ok(())
    }
}
