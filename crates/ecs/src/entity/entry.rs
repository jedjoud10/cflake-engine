use super::{Entity, EntityLinkings};
use crate::{Component, EcsManager, LayoutQuery, Query, QueryError};

// An entity entry that we can use to access multiple components on a single entity
pub struct EntityEntry<'a, Layout: LayoutQuery> {
    // Specific entity linkings
    linkings: EntityLinkings,

    // Internal query builder for fetching components
    builder: Query<'a, Layout>,
}

impl<'a, Layout: LayoutQuery> EntityEntry<'a, Layout> {
    // Create self from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Fetch the entity linkings and it's state
        let linkings = *manager.entities.get(entity).and_then(|x| x.as_ref())?;

        // Check if the linkings belong to a valid entity
        if !linkings.is_valid() {
            return None;
        }

        let mask = linkings.mask;
        Some(Self {
            linkings,
            builder: Query::new_from_mask(manager, mask),
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
