use super::{registry, Component, ComponentReadGuard, ComponentWriteGuard, EnclosedComponent, EnclosedGlobalComponent, GlobalComponent, GlobalComponentReadGuard, GlobalComponentWriteGuard};
use crate::{entity::Entity, utils::ComponentError};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::simple::OrderedVec;
use std::cell::UnsafeCell;

// Some global components that are stored in the ECS manager
#[derive(Clone)]
pub struct StoredGlobalComponents {
    // Our stored global components
    pub(crate) global_components: AHashMap<Bitfield<u32>, *mut EnclosedGlobalComponent>,
}

impl StoredGlobalComponents {
    // Create some stored global components using the global_component_access_cbitfield of a specific system and the global components
    pub(crate) fn new(global_component_access_cbitfield: Bitfield<u32>, global_components: &AHashMap<Bitfield<u32>, UnsafeCell<EnclosedGlobalComponent>>) -> Self {
        let global_components = global_components.iter().filter_map(|(bitfield, global_component)| {
            if global_component_access_cbitfield.contains(&bitfield) {
                let ptr = global_component.get();
                Some((*bitfield, ptr))
            } else { None }
        }).collect::<AHashMap<_, _>>();
        Self { global_components }
    }
}

impl StoredGlobalComponents {
    // Get a reference to a specific global component
    pub fn global_component<'b, T>(&self) -> Result<GlobalComponentReadGuard<'b, T>, ComponentError>
    where
        T: GlobalComponent + Send + Sync + 'static,
    {
        let id = registry::get_global_component_bitfield::<T>();
        // Kill me
        let hashmap = &self.global_components;
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast_global_component::<T>(component)?;
        let guard = GlobalComponentReadGuard::new(component);
        Ok(guard)
    }
    // Get a mutable reference to a specific global component
    pub fn globalcomponent_mut<'b, T>(&mut self) -> Result<GlobalComponentWriteGuard<'b, T>, ComponentError>
    where
        T: GlobalComponent + Send + Sync + 'static,
    {
        let id = registry::get_global_component_bitfield::<T>();
        let hashmap = &self.global_components;
        let ptr = *hashmap
            .get(&id)
            .ok_or_else(|| ComponentError::new_without_id("Linked component could not be fetched!".to_string()))?;
        // Magic
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_global_component_mut::<T>(component)?;
        let guard = GlobalComponentWriteGuard::new(component);
        Ok(guard)
    }
}
