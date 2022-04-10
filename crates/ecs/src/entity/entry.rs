use std::marker::PhantomData;

use super::Entity;
use crate::{registry, Archetype, Component, EcsManager, EntityEntryError};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a> {
    // Internal query for fetching components
    //query: EntityEntryQuery<'a>,
    archetype: &'a Archetype,
    bundle: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> EntityEntry<'a> {
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(_manager: &'a mut EcsManager, _entity: Entity) -> Option<Self> {
        //EntityEntryQuery::new(manager, entity).map(|query| Self { query })
        todo!()
    }
    // Get a pointer to a linked component of this entity
    pub unsafe fn get_ptr<T: Component>(&self) -> Result<*mut T, EntityEntryError> {
        let mask = registry::mask::<T>().map_err(EntityEntryError::ComponentError)?;
        let column = &self.archetype.vectors[&mask];
        let ptr = column.get_ptr() as *mut T;
        Ok(ptr.add(self.bundle))
    }
    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntityEntryError> {
        unsafe { self.get_ptr::<T>().map(|ptr| &*ptr) }
    }
    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntityEntryError> {
        unsafe { self.get_ptr::<T>().map(|ptr| &mut *ptr) }
    }
}
