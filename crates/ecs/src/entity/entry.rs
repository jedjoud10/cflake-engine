use super::{Entity, EntityLinkings};
use crate::{Component, EcsManager, LayoutQuery, Query, QueryError, EntityEntryQuery};

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
    /*
    // Create an immutable component
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        //self.query.
    }
    // Create a mutable component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
    }
    */
}