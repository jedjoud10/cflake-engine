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
    pub id: (u64, bool),
}

unsafe impl Sync for LinkedComponents {}
unsafe impl Send for LinkedComponents {}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new(entity: &Entity, mutated_components: Arc<AtomicSparseBitfield>, components: ComponentsCollection) -> Self {
        Self {
            components,
            mutated_components,
            linked: entity.components.clone(),
            id: (entity.id.unwrap().0, true),
        }
    }

    pub(crate) fn new_direct(
        id: EntityID,
        linked: &AHashMap<Bitfield<u32>, u64>,
        mutated_components: Arc<AtomicSparseBitfield>,
        components: ComponentsCollection,
    ) -> Self {
        Self {
            components,
            mutated_components,
            linked: linked.clone(),
            id: (id.0, true),
        }
    }

    pub(crate) fn new_dead(
        id: u64,
        linked: &AHashMap<Bitfield<u32>, u64>,
        mutated_components: Arc<AtomicSparseBitfield>,
        components: ComponentsCollection,
    ) -> Self {
        Self {
            components,
            mutated_components,
            linked: linked.clone(),
            id: (id, false),
        }
    }
}

// Function that create the "Linked component could not be fetched!" error
fn invalid_err() -> ComponentError {
    ComponentError::new_without_id("Linked component could not be fetched!".to_string())
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
    // Get a reference to a specific linked component
    pub fn get_component<T>(&self) -> Result<ComponentReadGuard<T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get_component_bitfield::<T>();
        let id = self.linked.get(&cbitfield).ok_or_else(invalid_err)?;
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
        let id = self.linked.get(&cbitfield).ok_or_else(invalid_err)?;
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
