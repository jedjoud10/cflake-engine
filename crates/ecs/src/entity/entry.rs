use super::Entity;
use crate::{Component, EcsManager, EntityEntryQuery, QueryError, EntityState};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a> {
    // Internal query for fetching components
    query: EntityEntryQuery<'a>,
}

impl<'a> EntityEntry<'a> {
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        EntityEntryQuery::new(manager, entity).map(|query| Self { query })
    }
    // Certified wrapper moment
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        self.query.get()
    }
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        self.query.get_mut()
    }
    pub fn was_mutated<T: Component>(&self) -> Result<bool, QueryError> {
        self.query.was_mutated::<T>()
    }
    pub fn state(&self) -> EntityState {
        self.query.entity_state()
    }
}
