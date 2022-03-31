use super::{Entity, EntityLinkings};
use crate::{Component, EcsManager, QueryBuilder, QueryError};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a> {
    // Specific entity linkings
    linkings: EntityLinkings,

    // Internal query builder for fetching components
    builder: QueryBuilder<'a>,
}

impl<'a> EntityEntry<'a> {
    // Create self from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Fetch the entity linkings and it's state
        let linkings = manager.entities.get(entity).and_then(|x| x.as_ref())?.clone();

        // Check if the linkings belong to a valid entity
        if !linkings.is_valid() {
            return None;
        }

        let mask = linkings.mask;
        Some(Self {
            linkings,
            builder: QueryBuilder::new(manager, mask),
        })
    }
    // Create an immutable component
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        // Get a pointer to the data and deref it
        let ptr = self.builder.get_ptr::<T>(self.linkings.bundle, self.linkings.mask)?;
        Ok(unsafe { &*ptr })
    }
    // Create a mutable component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        // Get a pointer to the data and deref it
        let ptr = self.builder.get_ptr::<T>(self.linkings.bundle, self.linkings.mask)?;
        Ok(unsafe { &mut *ptr })
    }
}
