use std::cell::RefCell;

use crate::{cast_component, cast_component_mut, component_registry, Component, ComponentError, ComponentID, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent, EntityID};
use ahash::AHashMap;
use bitfield::Bitfield;
// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents {
    components: AHashMap<Bitfield<u32>, *mut EnclosedComponent>,
    pub(crate) entity_id: EntityID, 
}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new(id: &EntityID, components: &AHashMap<ComponentID, RefCell<EnclosedComponent>>, cbitfield: &Bitfield<u32>) -> Self {
        // Get the components from the world, that fit the cbitfield and the Entity ID
        let filtered_components = components
            .iter()
            .filter_map(|(component_id, component)| {
                if cbitfield.contains(&component_id.cbitfield) && component_id.entity_id == *id {
                    // The component is linked to the entity, and we must get the component's pointer
                    let ptr = component.as_ptr();
                    Some((component_id.cbitfield, ptr))
                } else {
                    // The component is not linked to the entity
                    None
                }
            })
            .collect::<AHashMap<Bitfield<u32>, *mut EnclosedComponent>>();
        Self { components: filtered_components, entity_id: id.clone() }
    }
}

impl LinkedComponents {
    // Get a reference to a specific linked component
    pub fn component<'a, T>(&'a self) -> Result<ComponentReadGuard<'a, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let id = component_registry::get_component_bitfield::<T>();
        let ptr = *self
            .components
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &*ptr }.as_ref();
        let component = cast_component::<T>(component)?;
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn component_mut<'a, T>(&'a mut self) -> Result<ComponentWriteGuard<'a, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let id = component_registry::get_component_bitfield::<T>();
        // TODO: Make each entity have a specified amount of components so we can have faster indexing using
        // entity_id * 16 + local_component_id
        let ptr = *self
            .components
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &mut *ptr }.as_mut();
        let component = cast_component_mut::<T>(component)?;
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}