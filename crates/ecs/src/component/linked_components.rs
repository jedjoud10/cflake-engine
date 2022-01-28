use super::{registry, Component, ComponentID, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent};
use crate::{
    entity::{Entity, EntityID},
    utils::ComponentError,
    ECSManager,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, RwLock},
};

// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents {
    // Our linked components
    pub(crate) components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>,
    pub(crate) linked: AHashMap<Bitfield<u32>, u64>,

    // This ID can either be the valid entity ID or the ID of a removed entity that is stored in our temporary OrderedVec
    pub id: (u64, bool),
}

unsafe impl Sync for LinkedComponents {}
unsafe impl Send for LinkedComponents {}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new(entity: &Entity, components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>) -> Self {
        Self {
            components,
            linked: entity.components.clone(),
            id: (entity.id.unwrap().0, true),
        }
    }

    pub(crate) fn new_direct(id: EntityID, linked: &AHashMap<Bitfield<u32>, u64>, components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>) -> Self {
        Self {
            components,
            linked: linked.clone(),
            id: (id.0, true),
        }
    }

    pub(crate) fn new_dead(id: u64, linked: &AHashMap<Bitfield<u32>, u64>, components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>) -> Self {
        Self {
            components,
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
    // Get the entity ID of our corresponding entity
    pub fn get_entity_id(&self) -> Option<EntityID> {
        if !self.id.1 {
            return None;
        }
        Some(EntityID(self.id.0))
    }
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
    pub fn component<'b, T>(&self) -> Result<ComponentReadGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get_component_bitfield::<T>();
        let idx = self.linked.get(&cbitfield).ok_or(invalid_err())?;
        let hashmap = self.components.read().map_err(|_| invalid_err())?;
        let cell = hashmap.get(*idx).ok_or_else(|| invalid_err())?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast_component::<T>(component)?;

        // And now we've got a read guard!
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific linked entity components struct
    pub fn component_mut<'b, T>(&mut self) -> Result<ComponentWriteGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get_component_bitfield::<T>();
        let idx = self.linked.get(&cbitfield).ok_or(invalid_err())?;
        let hashmap = self.components.read().map_err(|_| invalid_err())?;
        let cell = hashmap.get(*idx).ok_or_else(|| invalid_err())?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_component_mut::<T>(component)?;

        // And now we've got a write guard!
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}
