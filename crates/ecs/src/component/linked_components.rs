use super::{registry, Component, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent};
use crate::{
    entity::{Entity},
    utils::ComponentError,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use std::{
    cell::{UnsafeCell},
};


// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents {
    // Our linked components
    pub(crate) components: AHashMap<Bitfield<u32>, *mut EnclosedComponent>,
}

unsafe impl Send for LinkedComponents {}
unsafe impl Sync for LinkedComponents {}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new(entity: &Entity, components: &OrderedVec<UnsafeCell<EnclosedComponent>>) -> Self {
        // Get the components from the world, that fit the cbitfield and the Entity ID
        let filtered_components = entity
            .components
            .iter()
            .filter_map(|component_id| {
                // The component is linked to the entity, and we must get the component's pointer
                let component = &*components.get(component_id.id).unwrap();
                let ptr = component.get();
                Some((component_id.cbitfield, ptr))
            })
            .collect::<AHashMap<Bitfield<u32>, *mut EnclosedComponent>>();
        Self { components: filtered_components }
    }
}

impl LinkedComponents {
    // Get a reference to a specific linked component
    pub fn component<'b, T>(&self) -> Result<ComponentReadGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let id = registry::get_component_bitfield::<T>();
        // Kill me
        let hashmap = &self.components;
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast_component::<T>(component)?;
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn component_mut<'b, T>(&mut self) -> Result<ComponentWriteGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let id = registry::get_component_bitfield::<T>();
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        // Kill me
        let hashmap = &self.components;
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_component_mut::<T>(component)?;
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}
