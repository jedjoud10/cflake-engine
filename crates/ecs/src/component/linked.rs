use super::{registry, Component, ComponentKey, Components};
use crate::{entity::EntityKey, utils::ComponentError};
use ahash::AHashMap;
use bitfield::{AtomicSparseBitfield, Bitfield};
use getset::Getters;
use slotmap::Key;
use std::sync::Arc;

// Some linked components that we can mutate or read from in each system
// These components are stored on the main thread however
#[derive(Getters)]
pub struct LinkedComponents {
    // Our linked components
    pub(crate) components: Components,
    pub(crate) mutated_components: Arc<AtomicSparseBitfield>,
    pub(crate) linked: AHashMap<Bitfield<u32>, ComponentKey>,
    #[getset(get = "pub")]
    pub(crate) key: EntityKey,
}

unsafe impl Sync for LinkedComponents {}
unsafe impl Send for LinkedComponents {}

// Errors
fn invalid_err() -> ComponentError {
    ComponentError::new("Linked component could not be fetched!".to_string())
}
fn invalid_err_not_linked() -> ComponentError {
    ComponentError::new("Component is not linked to the entity!".to_string())
}
impl LinkedComponents {
    // Get a reference to a specific linked component
    pub fn get<T>(&self) -> Result<&T, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get::<T>();
        let key = self.linked.get(&cbitfield).ok_or_else(invalid_err_not_linked)?;
        let map = self.components.read();
        let cell = map.get(*key).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &*ptr }.as_ref();
        let component = registry::cast::<T>(component)?;
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    // This mutation can be detected using was_mutated
    pub fn get_mut<T>(&mut self) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get::<T>();
        let key = self.linked.get(&cbitfield).ok_or_else(invalid_err_not_linked)?;
        let map = self.components.read();
        let cell = map.get(*key).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_mut::<T>(component)?;
        // We only care about the index
        let index = key.data().as_ffi() & 0xffff_ffff;
        self.mutated_components.set(index as usize, true);
        Ok(component)
    }
    // Get a mutable reference to a specific linked entity components struct
    // This won't update the mutated_components bitfield
    pub fn get_mut_silent<T>(&mut self) -> Result<&mut T, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Get the UnsafeCell
        let cbitfield = registry::get::<T>();
        let key = self.linked.get(&cbitfield).ok_or_else(invalid_err_not_linked)?;
        let map = self.components.read();
        let cell = map.get(*key).ok_or_else(invalid_err)?;

        // Then get it's pointer and do black magic
        let ptr = cell.get();
        let component = unsafe { &mut *ptr }.as_mut();
        let component = registry::cast_mut::<T>(component)?;
        Ok(component)
    }
    // Check if a specific component has been updated during this frame
    pub fn was_mutated<T>(&self) -> Result<bool, ComponentError>
    where
        T: Component + Send + Sync,
    {
        // Check if we even have the component
        let cbitfield = registry::get::<T>();
        let key = self.linked.get(&cbitfield).ok_or_else(invalid_err)?;

        // We only care about the index
        let index = key.data().as_ffi() & 0xffff_ffff;
        Ok(self.mutated_components.get(index as usize))
    }
}
