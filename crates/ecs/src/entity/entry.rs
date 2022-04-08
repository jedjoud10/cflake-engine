use std::marker::PhantomData;

use super::Entity;
use crate::{Component, EcsManager, /*EntityEntryQuery*/ EntityState, QueryError};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a> {
    // Internal query for fetching components
    //query: EntityEntryQuery<'a>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> EntityEntry<'a> {
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        //EntityEntryQuery::new(manager, entity).map(|query| Self { query })
        todo!()
    }
    // Certified wrapper moment
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        //self.query.get()
        todo!()
    }
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        //self.query.get_mut()
        todo!()
    }
    pub fn was_mutated<T: Component>(&self) -> Result<bool, QueryError> {
        //self.query.was_mutated::<T>()
        todo!()
    }
    pub fn state(&self) -> EntityState {
        todo!()
        //self.query.entity_state()
    }
}
