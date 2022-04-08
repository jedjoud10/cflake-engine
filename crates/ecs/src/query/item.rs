use crate::{registry, Archetype, Component, ComponentError, Entity, Mask, QueryCache};

// Something that can be queried. This will be implement on Read<T> and Write<T> (where T is Component). This will also be implemented on Read<Entity>
pub trait QueryItem {
    // The item (either &T or &mut T) that will be given to the user
    type Item;

    // Try to get the mask of this query item (this might fail since Read<Entity> doesn't have a mask)
    fn item_mask() -> Result<Mask, ComponentError>;
    // Add the necessary data into the query cache
    fn cache(archetype: &Archetype, cache: &mut QueryCache);
}

impl<'a, T: Component> QueryItem for crate::Read<'a, T> {
    type Item = &'a T;
    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {}
}

impl<'a, T: Component> QueryItem for crate::Write<'a, T> {
    type Item = &'a mut T;

    fn item_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {}
}

impl<'a> QueryItem for crate::Read<'a, Entity> {
    type Item = &'a Entity;

    fn item_mask() -> Result<Mask, ComponentError> {
        Ok(Mask::default())
    }

    fn cache(archetype: &Archetype, cache: &mut QueryCache) {}
}
