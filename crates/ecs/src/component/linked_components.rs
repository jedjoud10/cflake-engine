use super::{registry, Component, ComponentID, ComponentReadGuard, ComponentWriteGuard, ComponentsCollection};
use crate::{
    entity::{Entity, EntityID},
    utils::ComponentError,
};
use ahash::AHashMap;
use bitfield::{AtomicSparseBitfield, Bitfield};
use std::sync::Arc;

// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents {
    // Our linked components
    pub(crate) components: ComponentsCollection,
    pub(crate) mutated_components: Arc<AtomicSparseBitfield>,
    pub(crate) linked: AHashMap<Bitfield<u32>, u64>,

    // This ID can either be the valid entity ID or the ID of a removed entity that is stored in our temporary OrderedVec
    pub id: u64,
}

unsafe impl Sync for LinkedComponents {}
unsafe impl Send for LinkedComponents {}

impl LinkedComponents {
    pub(crate) fn new(entity: &Entity, mutated_components: Arc<AtomicSparseBitfield>, components: ComponentsCollection) -> Self {
        Self {
            components,
            mutated_components,
            linked: entity.components.clone(),
            id: (entity.id.unwrap().0),
        }
    }

    pub(crate) fn new_direct(id: EntityID, linked: AHashMap<Bitfield<u32>, u64>, mutated_components: Arc<AtomicSparseBitfield>, components: ComponentsCollection) -> Self {
        Self {
            components,
            mutated_components,
            linked,
            id: (id.0),
        }
    }
}

// Errors
fn invalid_err() -> ComponentError {
    ComponentError::new("Linked component could not be fetched!".to_string())
}
fn invalid_err_not_linked() -> ComponentError {
    ComponentError::new("Component is not linked to the entity!".to_string())
}
impl LinkedComponents {
    // Get the component ID of a specific component that this entity has
    pub fn get_component_id<T>(&self) -> Option<ComponentID>
    where
        T: Component + Send + Sync + 'static,
    {
        let cbitfield = registry::get_component_bitfield::<T>();
        let idx = self.linked.get(&cbitfield)?;
        Some(ComponentID::new(cbitfield, *idx))
    }
    // Get the entity ID
    pub fn get_entity_id(&self) -> EntityID {
        EntityID(self.id)
    }
    // Get a reference to a specific linked component
    pub fn get_component<T>(&self) -> Result<ComponentReadGuard<T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get_component_bitfield::<T>();
        let id = self.linked.get(&cbitfield).ok_or_else(invalid_err_not_linked)?;
        let ordered_vec = self.components.read();
        let cell = ordered_vec.get(*id).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast_component::<T>(component)?;

        // And now we've got a read guard!
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn get_component_mut<T>(&mut self) -> Result<ComponentWriteGuard<T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get_component_bitfield::<T>();
        let id = self.linked.get(&cbitfield).ok_or_else(invalid_err_not_linked)?;
        let ordered_vec = self.components.read();
        let cell = ordered_vec.get(*id).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_component_mut::<T>(component)?;

        // And now we've got a write guard!
        let guard = ComponentWriteGuard::new(component);
        let index = ordered_vec::utils::from_id(*id).index;
        self.mutated_components.set(index.try_into().unwrap(), true);
        Ok(guard)
    }
    // Check if a specific component has been updated during this frame
    pub fn was_mutated<T>(&self) -> Result<bool, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Check if we even have the component
        let cbitfield = registry::get_component_bitfield::<T>();
        let id = self.linked.get(&cbitfield).ok_or_else(invalid_err)?;

        // Now check if it has been mutated or not
        let index = ordered_vec::utils::from_id(*id).index;
        Ok(self.mutated_components.get(index.try_into().unwrap()))
    }
}
