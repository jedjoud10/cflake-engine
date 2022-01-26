use super::{registry, Component, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent, ComponentID};
use crate::{
    entity::{Entity, EntityID},
    utils::ComponentError, ECSManager,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use std::{cell::UnsafeCell, sync::{Arc, Mutex, RwLock}};

// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
pub struct LinkedComponents {
    // Our linked components
    pub(crate) components: Arc<RwLock<OrderedVec<UnsafeCell<EnclosedComponent>>>>,
    pub(crate) linked: *const AHashMap<Bitfield<u32>, u64>,
    pub id: EntityID,
}

unsafe impl Sync for LinkedComponents {}
unsafe impl Send for LinkedComponents {}

impl LinkedComponents {
    // Create some linked components from an Entity ID, the full AHashMap of components, and the System cbitfield
    // Theoretically, this should only be done once, when an entity becomes valid for a system
    pub(crate) fn new<Context>(entity: &Entity, ecs_manager: &ECSManager<Context>) -> Self {
        Self {
            components: ecs_manager.components.clone(),
            linked: &entity.components as *const _,
            id: entity.id.unwrap(),
        }
    }
}

// Function that create the "Linked component could not be fetched!" error
fn invalid_err() -> ComponentError {
    ComponentError::new_without_id("Linked component could not be fetched!".to_string())
}
impl LinkedComponents {
    // Get a reference to a specific linked component
    pub fn component<'b, T>(&self) -> Result<ComponentReadGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {        
        // Get the UnsafeCell
        let id = registry::get_component_bitfield::<T>();
        let idx = unsafe { &*self.linked }.get(&id).ok_or(invalid_err())?;
        let hashmap = self.components.read().map_err(|_| invalid_err())?;
        let cell = hashmap
            .get(*idx)
            .ok_or_else(|| invalid_err())?;

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
        let id = registry::get_component_bitfield::<T>();
        let idx = unsafe { &*self.linked }.get(&id).ok_or(invalid_err())?;
        let hashmap = self.components.read().map_err(|_| invalid_err())?;
        let cell = hashmap
            .get(*idx)
            .ok_or_else(|| invalid_err())?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_component_mut::<T>(component)?;

        // And now we've got a write guard!
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}
