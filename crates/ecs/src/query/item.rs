use crate::{Archetype, QueryCache, Component, Entity, ComponentError, registry, Mask};

// Something that can be queried. This will be implement on Read<T> and Write<T> (where T is Component). This will also be implemented on Read<Entity>
pub trait QueryItem {
    // Try to get the mask of this query item (this might fail since Read<Entity> doesn't have a mask)
    fn item_mask() -> Result<Mask, ComponentError>;
    // Add the necessary data into the query cache
    fn cache(archetype: &Archetype, cache: &mut QueryCache);
}

impl<T: Component> QueryItem for crate::Read<'_, T> {
    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {
    }
}

impl<T: Component> QueryItem for crate::Write<'_, T> {
    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {
    }
}

impl QueryItem for crate::Read<'_, Entity> {
    fn item_mask() -> Result<Mask, ComponentError> {
        Ok(Mask::default())
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {
    }
}