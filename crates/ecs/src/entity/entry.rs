use std::marker::PhantomData;

use super::Entity;
use crate::{registry, Archetype, Component, EcsManager, EntityEntryError};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a> {
    // Internal query for fetching components
    //query: EntityEntryQuery<'a>,
    archetype: &'a Archetype,
    bundle: usize,
}

impl<'a> EntityEntry<'a> {
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        let linkings = manager.entities.get(entity)?;
        Some(Self {
            archetype: manager.archetypes.get(&linkings.mask)?,
            bundle: linkings.bundle,
        })
    }
    // Get a pointer to a linked component of this entity
    pub unsafe fn get_ptr<T: Component>(&self) -> Result<*mut T, EntityEntryError> {
        let mask = registry::mask::<T>().map_err(EntityEntryError::ComponentError)?;
        let (_, ptr) = &self.archetype.vectors[&mask];
        let ptr = *ptr as *mut T;
        Ok(ptr.add(self.bundle))
    }
    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntityEntryError> {
        unsafe { self.get_ptr::<T>().map(|ptr| &*ptr) }
    }
    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntityEntryError> {
        // Update the mutation state
        let mask = registry::mask::<T>().map_err(EntityEntryError::ComponentError)?;
        self.archetype.states.set(self.bundle, mask);

        unsafe { self.get_ptr::<T>().map(|ptr| &mut *ptr) }
    }
}
