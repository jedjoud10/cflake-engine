use super::{registry, Component, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent, EnclosedGlobalComponent};
use crate::{entity::Entity, utils::ComponentError, ECSManager};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use std::{cell::UnsafeCell, sync::{Mutex, Arc}};

// Some global components that are stored in the ECS manager
#[derive(Clone)]
pub struct StoredGlobalComponents {
    // Our stored global components
    pub(crate) global_components: Arc<Mutex<AHashMap<Bitfield<u32>, UnsafeCell<EnclosedGlobalComponent>>>>,
    pub(crate) global_access_cbitfield: Bitfield<u32>,
}

impl StoredGlobalComponents {
    // Create some stored global components using the global_component_access_cbitfield of a specific system and the global components
    pub(crate) fn new<Context>(global_access_cbitfield: Bitfield<u32>, ecs_manager: &ECSManager<Context>) -> Self {
        Self {
            global_access_cbitfield,
            global_components: ecs_manager.global_components.clone(),
        }
    }
}

impl StoredGlobalComponents {
    // Get a reference to a specific global component
    pub fn global_component<'b, T>(&self) -> Result<ComponentReadGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let id = registry::get_component_bitfield::<T>();
        // Kill me
        let hashmap = &self.global_components;
        let hashmap = hashmap.lock().unwrap();
        let ptr = hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &*ptr.get() }.as_ref();
        let component = registry::cast_component::<T>(component)?;
        let guard = ComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific global component
    pub fn global_component_mut<'b, T>(&mut self) -> Result<ComponentWriteGuard<'b, T>, ComponentError>
    where
        T: Component + Send + Sync + 'static,
    {
        let id = registry::get_component_bitfield::<T>();
        let hashmap = &self.global_components;
        let hashmap = hashmap.lock().unwrap();
        let ptr = hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &mut *ptr.get() }.as_mut();
        let component = registry::cast_component_mut::<T>(component)?;
        let guard = ComponentWriteGuard::new(component);
        Ok(guard)
    }
}
