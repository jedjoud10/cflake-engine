use std::any::TypeId;

use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{
    component::{registry, Component, EnclosedComponent},
    utils::ComponentLinkingError,
};

use super::Entity;
// A collection of components that will be mass linked to a specific entity when it gets added into the world on the main thread
#[derive(Default)]
pub struct ComponentLinkingGroup {
    pub linked_components: AHashMap<Bitfield<u32>, EnclosedComponent>,
    pub cbitfield: Bitfield<u32>,
}

// Linking methods
impl ComponentLinkingGroup {
    // Link a component to this entity and automatically set it to the default variable
    pub fn link_default<T: Component + Send + Sync + Default + 'static>(&mut self) -> Result<(), ComponentLinkingError> {
        // Simple wrapper around the default link component
        self.link(T::default())
    }
    // Link a component to this entity
    pub fn link<T: Component + Send + Sync + 'static>(&mut self, default_state: T) -> Result<(), ComponentLinkingError> {
        let cbitfield = registry::get_component_bitfield::<T>();
        // Check if we have the component linked on this linking group
        if let std::collections::hash_map::Entry::Vacant(e) = self.linked_components.entry(cbitfield) {
            // Add the local component to our hashmap
            let boxed = Box::new(default_state);
            e.insert(boxed);
        } else {
            // The component was already linked to the group
            return Err(ComponentLinkingError::new(format!(
                "Cannot link component '{:?}' to ComponentLinkingGroup because it is already linked to the group!",
                TypeId::of::<T>(),
            )));
        }
        // Add the component's bitfield to the entity's bitfield
        self.cbitfield = self.cbitfield.add(&cbitfield);
        Ok(())
    }
}

// A collection of omponents that we will remove from the entity
#[derive(Default)]
pub struct ComponentUnlinkGroup {
    pub(crate) removal_cbitfield: Bitfield<u32>,
}

// Linking methods
impl ComponentUnlinkGroup {
    // Unlink a component from the entity
    pub fn unlink<T: Component + 'static>(&mut self) -> Result<(), ComponentLinkingError> {
        self.removal_cbitfield = self.removal_cbitfield.add(&registry::get_component_bitfield::<T>());
        Ok(())
    }
    // Unlink all the components from an entity
    pub fn unlink_all_from_entity(entity: &Entity) -> Self {
        Self {
            removal_cbitfield: entity.cbitfield,
        }
    }
}
